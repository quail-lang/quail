use std::collections::HashMap;

use dirs;
use rustyline::error::ReadlineError;

use crate::parser;
use crate::ast;
use crate::hole;
use crate::builtins;

use ast::Term;
use ast::TermNode;
use ast::Def;
use ast::Value;
use ast::Context;
use ast::HoleId;
use ast::Import;
use ast::MatchArm;

#[derive(Debug)]
pub struct Runtime {
    pub imports: Vec<String>,
    pub definitions: Vec<Def>,
    pub holes: HashMap<HoleId, Value>,
    pub readline_file: String,
    pub editor: rustyline::Editor<()>,
    pub builtin_ctx: Context,
}

impl Runtime {
    pub fn load(filepath: impl AsRef<std::path::Path>) -> Self {
        let readline_file = dirs::config_dir()
            .expect("User does not have a home directory??")
            .join("quail").join("history");

        println!("Loading readline history: {:?}", &readline_file);
        if !readline_file.exists() {
            std::fs::create_dir_all(&readline_file.parent().unwrap()).unwrap();
            std::fs::File::create(&readline_file).expect("Could not create readline file");
        }

        let mut runtime = Runtime {
            imports: vec![],
            definitions: vec![],
            holes: HashMap::new(),
            readline_file: readline_file.to_string_lossy().to_string(),
            editor: rustyline::Editor::new(),
            builtin_ctx: builtins::builtins_ctx(),
        };

        if runtime.editor.load_history(&runtime.readline_file).is_err() {
            eprintln!("Could not read from {:?} for readline history.", &readline_file);
        }

        let basedir = std::path::Path::new(filepath.as_ref().parent().expect("Invalid path"));
        let filename = std::path::Path::new(filepath.as_ref().file_name().expect("Invalid path"));
        runtime.load_module(filename, basedir, true);
        runtime
    }

    fn load_module(&mut self, filename: &std::path::Path, basedir: &std::path::Path, is_main: bool) {
        if self.imports.contains(&filename.to_string_lossy().to_string()) {
            return;
        } else {
            self.imports.push(filename.to_string_lossy().to_string());
        }
        let filepath = basedir.join(filename);
        println!("Loading {:?}", filepath.to_string_lossy());
        use std::fs;
        use std::io::Read;
        let mut module_text = String::new();
        fs::File::open(&filepath)
            .unwrap_or_else(|e| panic!(format!("There was an error when opening {:?}: {:?}", &filepath, e)))
            .read_to_string(&mut module_text)
            .unwrap_or_else(|e| panic!(format!("There was an error {:?}", e)));

        let module = parser::parse_module(&module_text)
            .unwrap_or_else(|e| panic!(format!("There was an error {:?}", e)));

        if is_main {
            self.definitions.extend(module.definitions);
        } else {
            self.definitions.extend(module.definitions.into_iter().filter(|Def(name, _)| name != "main"));
        }
        for import in module.imports {
            let Import(name) = import;

            let mut import_filename = name.to_string();
            import_filename.push_str(".ql");
            let import_filename = &std::path::Path::new(&import_filename);

            self.load_module(import_filename, basedir, false);
        }
    }

    fn definition(&self, name: &str) -> Option<&Def> {
        for definition in &self.definitions {
            let Def(def_name, _) = &definition;
            if def_name == name {
                return Some(definition);
            }
        }
        None
    }

    pub fn exec(&mut self) {
        let Def(_, main_body) = self.definition("main").expect("There should be a main in your module").clone();
        self.eval(main_body, Context::empty());
    }

    pub fn readline(&mut self) -> Result<String, ReadlineError> {
        let line = self.editor.readline("> ")?;
        self.editor.add_history_entry(line.as_str());
        self.editor.save_history(&self.readline_file)?;
        Ok(line)
    }

    pub fn fill_hole(&mut self, hole_id: HoleId, value: Value) {
        self.holes.insert(hole_id, value);
    }

    pub fn lookup_builtin(&self, x: &str) -> Option<Value> {
        self.builtin_ctx.lookup(x)
    }

    fn apply(self: &mut Runtime, func: Value, args: Vec<Value>) -> Value {
        match &func {
            Value::Fun(x, body, local_ctx) => {
                match args.clone().split_first() {
                    None => func,
                    Some((v, vs_remaining)) => {
                        let new_ctx = local_ctx.extend(&x, v.clone());
                        let new_func = self.eval(body.clone(), new_ctx);
                        self.apply(new_func, vs_remaining.to_vec())
                    },
                }
            },
            Value::Ctor(tag, contents) => {
                let mut new_contents = contents.clone();
                new_contents.extend(args);
                Value::Ctor(tag.to_string(), new_contents)
            },
            Value::Prim(prim) => {
                prim(args)
            },
            _ => panic!(format!("Applied arguments to non-function {:?}", func)),
        }
    }

    #[allow(mutable_borrow_reservation_conflict)]
    pub fn eval(self: &mut Runtime, t: Term, ctx: Context) -> Value {
        match t.as_ref() {
            TermNode::Var(x) => {
                match self.definition(x) {
                    Some(Def(_, body)) => self.eval(body.clone(), ctx),
                    None => {
                        match ctx.lookup(&x) {
                            Some(v) => v,
                            None => {
                                match self.lookup_builtin(&x) {
                                    Some(v) => v,
                                    None => panic!(format!("Unbound variable {:?}", &x)),
                                }
                            },
                        }
                    }
                }
            },
            TermNode::Lam(x, body) => {
                Value::Fun(x.clone(), body.clone(), ctx.clone())
            },
            TermNode::App(f, vs) => {
                let f_value = self.eval(f.clone(), ctx.clone());
                let vs_values: Vec<Value> = vs.iter().map(|v| self.eval(v.clone(), ctx.clone())).collect();
                self.apply(f_value, vs_values)
            },
            TermNode::Let(x, v, body) => {
                let v_value = self.eval(v.clone(), ctx.clone());
                let extended_ctx = ctx.extend(x, v_value);
                self.eval(body.clone(), extended_ctx)
            },
            TermNode::Match(t, match_arms) => {
                let t_value = self.eval(t.clone(), ctx.clone());
                match t_value {
                    Value::Ctor(tag, contents) => {
                        let MatchArm(pat, body) = ast::find_matching_arm(&tag, &match_arms);

                        let bind_names: Vec<String> = pat[1..].to_vec();
                        let bind_values: Vec<Value> = contents.clone();
                        let bindings: Vec<(String, Value)> = bind_names.into_iter().zip(bind_values).collect();

                        let extended_ctx = ctx.extend_many(&bindings);
                        self.eval(body, extended_ctx)
                    },
                    _ => panic!(format!("Expected a constructor during match statement, but found {:?}", &t_value)),
                }
            },
            TermNode::Hole(hole_info) => hole::fill(self, hole_info, ctx),
        }
    }

}
