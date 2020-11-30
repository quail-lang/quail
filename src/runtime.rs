use std::collections::HashMap;

use dirs;
use rustyline::error::ReadlineError;

use crate::parser;
use crate::ast;
use crate::hole;
use crate::builtins;
use crate::typecheck;

use ast::Term;
use ast::TermNode;
use ast::Def;
use ast::Value;
use ast::Context;
use ast::HoleId;
use ast::Import;
use ast::MatchArm;
use builtins::InductiveTypeDef;
use typecheck::TypeContext;

#[derive(Debug)]
pub struct Runtime {
    pub imports: Vec<String>,
    pub holes: HashMap<HoleId, Value>,
    pub readline_file: String,
    pub editor: rustyline::Editor<()>,
    pub number_of_holes: u64,

    pub inductive_typedefs: HashMap<String, InductiveTypeDef>,

    pub definition_ctx: Context,
    pub definition_type_ctx: TypeContext,

    pub builtin_ctx: Context,
    pub builtin_type_ctx: TypeContext,
}

impl Runtime {
    pub fn new() -> Self {
        let readline_file = dirs::config_dir()
            .expect("User does not have a home directory??")
            .join("quail").join("history");

        if !readline_file.exists() {
            std::fs::create_dir_all(&readline_file.parent().unwrap()).unwrap();
            std::fs::File::create(&readline_file).expect("Could not create readline file");
        }

        let inductive_typedefs: HashMap<String, InductiveTypeDef> = builtins::builtin_inductive_typedefs()
            .iter()
            .map(|itd| (itd.name.to_string(), itd.clone()))
            .collect();

        let mut builtin_ctx = builtins::builtins_ctx();
        let mut builtin_type_ctx = builtins::builtins_type_ctx();

        for inductive_typedef in inductive_typedefs.values() {
            builtin_ctx = builtin_ctx.append(inductive_typedef.ctor_context());
            builtin_type_ctx = builtin_type_ctx.append(inductive_typedef.ctor_type_context());
        }

        let mut editor = rustyline::Editor::new();
        if editor.load_history(&readline_file).is_err() {
            eprintln!("Could not read from {:?} for readline history.", &readline_file);
        }

        Runtime {
            imports: vec![],
            holes: HashMap::new(),
            readline_file: readline_file.to_string_lossy().to_string(),
            editor,
            number_of_holes: 0,

            inductive_typedefs,

            definition_ctx: Context::empty(),
            builtin_ctx,

            definition_type_ctx: TypeContext::empty(),
            builtin_type_ctx,
        }
    }

    pub fn import(&mut self, name: &str) -> Result<(), RuntimeError> {
        let mut import_filename = name.to_string();
        import_filename.push_str(".ql");
        let import_filename = &std::path::Path::new(&import_filename);

        match std::env::current_dir() {
            Ok(basedir) => self.load_module(import_filename, &basedir, false),
            Err(err) => Err(err.into()),
        }
    }

    pub fn load(&mut self, filepath: impl AsRef<std::path::Path>) -> Result<(), RuntimeError> {
        let basedir = std::path::Path::new(filepath.as_ref().parent().expect("Invalid path"));
        let filename = std::path::Path::new(filepath.as_ref().file_name().expect("Invalid path"));
        self.load_module(filename, basedir, true)
    }

    fn load_module(
        &mut self,
        filename: &std::path::Path,
        basedir: &std::path::Path,
        is_main: bool,
    ) -> Result<(), RuntimeError> {
        if self.imports.contains(&filename.to_string_lossy().to_string()) {
            return Ok(());
        } else {
            self.imports.push(filename.to_string_lossy().to_string());
        }
        let filepath = basedir.join(filename);
        use std::fs;
        use std::io::Read;
        let mut module_text = String::new();
        fs::File::open(&filepath)?.read_to_string(&mut module_text)?;

        let (module, number_of_new_holes) = parser::parse_module(self.next_hole_id(), Some(filename), &module_text)?;
        self.add_holes(number_of_new_holes);

        for import in module.imports {
            let Import(name) = import;

            let mut import_filename = name.to_string();
            import_filename.push_str(".ql");
            let import_filename = &std::path::Path::new(&import_filename);

            self.load_module(import_filename, basedir, false)?;
        }

        for definition in module.definitions.iter() {
            let Def(name, typ, body) = definition;
            if is_main || name != "main" {
                let type_context = self.builtin_type_ctx.append(self.definition_type_ctx.clone()).extend(name, typ.clone());
                typecheck::check_type(body.clone(), type_context, &self.inductive_typedefs, typ.clone())?;
                self.definition_type_ctx = self.definition_type_ctx.extend(&name.to_string(), typ.clone());

                let body_value = self.eval(body.clone(), Context::empty());
                self.definition_ctx = self.definition_ctx.extend(&name.to_string(), body_value);
            }
        }
        Ok(())
    }

    pub fn define(&mut self, definition: &Def) -> Result<(), RuntimeError> {
        let Def(name, typ, body) = definition;
        let type_context = self.builtin_type_ctx.append(self.definition_type_ctx.clone()).extend(&name, typ.clone());
        typecheck::check_type(body.clone(), type_context, &self.inductive_typedefs, typ.clone())?;
        self.definition_type_ctx = self.definition_type_ctx.extend(&name.to_string(), typ.clone());

        let body_value = self.eval(body.clone(), Context::empty());
        self.definition_ctx = self.definition_ctx.extend(&name.to_string(), body_value);
        Ok(())
    }

    pub fn next_hole_id(&self) -> HoleId {
        self.number_of_holes as HoleId
    }

    pub fn add_holes(&mut self, number_of_holes: u64) {
        self.number_of_holes += number_of_holes;
    }

    pub fn exec(&mut self) {
        self.definition_ctx.lookup("main").expect("There should be a main in your module");
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

    pub fn hole_value(&self, hole_id: HoleId) -> Option<Value> {
        assert!((hole_id as u64) < self.number_of_holes, "Invalid HoleId!");
        self.holes.get(&hole_id).cloned()
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
                ctx.lookup(&x)
                    .or_else(|| self.definition_ctx.lookup(&x))
                    .or_else(|| self.builtin_ctx.lookup(&x))
                    .expect(&format!("Unbound variable {:?}", &x))
            },
            TermNode::StrLit(contents) => Value::Str(contents.to_string()),
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
            TermNode::As(term, _typ) => self.eval(term.clone(), ctx),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError(String);

impl std::convert::From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        RuntimeError(error.to_string())
    }
}

impl std::convert::From<String> for RuntimeError {
    fn from(error: String) -> Self {
        RuntimeError(error)
    }
}
