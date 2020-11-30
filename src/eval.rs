use std::collections::HashMap;
use std::rc;

use dirs;
use rustyline::error::ReadlineError;

use crate::parser;
use crate::ast;
use crate::hole;

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
            builtin_ctx: builtins_ctx(),
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
        eval(main_body, Context::empty(), self);
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
}

fn succ_prim(vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "succ must have exactly one argument");
    let v = vs[0].clone();
    match &v {
        Value::Ctor(tag, _) => {
            if tag == "zero" {
                Value::Ctor("succ".into(), vec![Value::Ctor("zero".into(), vec![])])
            } else if tag == "succ" {
                Value::Ctor("succ".into(), vec![v.clone()])
            } else {
                panic!("Invalid thing to succ: {:?}", &v)
            }
        },
        other => panic!(format!("Couldn't succ {:?}", other)),
    }
}

fn cons_prim(vs: Vec<Value>) -> Value {
    let head = vs[0].clone();
    let tail = vs[1].clone();
    Value::Ctor("cons".into(), vec![head, tail])
}

fn pair_prim(vs: Vec<Value>) -> Value {
    let fst = vs[0].clone();
    let snd = vs[1].clone();
    Value::Ctor("pair".into(), vec![fst, snd])
}

fn println_prim(vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "println must have exactly one argument");
    let v = vs[0].clone();
    println!("{:?}", v);
    Value::Ctor("unit".into(), Vec::new())
}

fn nat_to_u64(v: Value) -> u64 {
    match v {
        Value::Ctor(tag, contents) => {
            if tag == "zero" {
                0
            } else if tag == "succ" {
                let inner_value = &contents[0];
                1 + nat_to_u64(inner_value.clone())
            } else {
                 panic!("This isn't a nat.")
            }
        },
        _ => panic!("This isn't a nat."),
    }
}

fn list_to_vec(v: Value) -> Vec<Value> {
    match v {
        Value::Ctor(tag, contents) => {
            if tag == "nil" {
                Vec::new()
            } else if tag == "cons" {
                let head = &contents[0];
                let tail = &contents[1];
                let mut result = list_to_vec(tail.clone());
                result.insert(0, head.clone());
                result
            } else {
                 panic!("This isn't a list.")
            }
        },
        _ => panic!("This isn't a list."),
    }
}

fn show_prim(vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "show must have exactly one argument");
    let v = vs[0].clone();
    match &v {
        Value::Ctor(tag, _) => {
            if tag == "zero" || tag == "succ" {
                Value::Str(format!("{}", nat_to_u64(v)))
            } else if tag == "nil" || tag == "cons" {
                let val_vec = list_to_vec(v.clone());
                let str_value_vec: Vec<Value> = val_vec.into_iter().map(|v| show_prim(vec![v])).collect();
                let s: String = format!("{:?}", str_value_vec);
                Value::Str(s)
            } else {
                Value::Str(format!("{:?}", v))
            }
        }
        _ => panic!("Can't show this {:?}", &v),
    }
}

pub fn builtins_ctx() -> Context {
    Context::empty()
        .extend("println", Value::Prim(rc::Rc::new(Box::new(println_prim))))
        .extend("zero", Value::Ctor("zero".into(), Vec::new()))
        .extend("succ", Value::Prim(rc::Rc::new(Box::new(succ_prim))))
        .extend("true", Value::Ctor("true".into(), Vec::new()))
        .extend("false", Value::Ctor("false".into(), Vec::new()))
        .extend("nil", Value::Ctor("nil".into(), Vec::new()))
        .extend("cons", Value::Prim(rc::Rc::new(Box::new(cons_prim))))
        .extend("unit", Value::Ctor("unit".into(), Vec::new()))
        .extend("pair", Value::Prim(rc::Rc::new(Box::new(pair_prim))))
        .extend("show", Value::Prim(rc::Rc::new(Box::new(show_prim))))
}

fn apply(func: Value, args: Vec<Value>, runtime: &mut Runtime) -> Value {
    match &func {
        Value::Fun(x, body, local_ctx) => {
            match args.clone().split_first() {
                None => func,
                Some((v, vs_remaining)) => {
                    let new_ctx = local_ctx.extend(&x, v.clone());
                    let new_func = eval(body.clone(), new_ctx, runtime);
                    apply(new_func, vs_remaining.to_vec(), runtime)
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

pub fn eval(t: Term, ctx: Context, runtime: &mut Runtime) -> Value {
    match t.as_ref() {
        TermNode::Var(x) => {
            match runtime.definition(x) {
                Some(Def(_, body)) => eval(body.clone(), ctx, runtime),
                None => {
                    match ctx.lookup(&x) {
                        Some(v) => v,
                        None => {
                            match runtime.lookup_builtin(&x) {
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
            let f_value = eval(f.clone(), ctx.clone(), runtime);
            let vs_values: Vec<Value> = vs.iter().map(|v| eval(v.clone(), ctx.clone(), runtime)).collect();
            apply(f_value, vs_values, runtime)
        },
        TermNode::Let(x, v, body) => {
            let v_value = eval(v.clone(), ctx.clone(), runtime);
            let extended_ctx = ctx.extend(x, v_value);
            eval(body.clone(), extended_ctx, runtime)
        },
        TermNode::Match(t, match_arms) => {
            let t_value = eval(t.clone(), ctx.clone(), runtime);
            match t_value {
                Value::Ctor(tag, contents) => {
                    let MatchArm(pat, body) = ast::find_matching_arm(&tag, &match_arms);

                    let bind_names: Vec<String> = pat[1..].to_vec();
                    let bind_values: Vec<Value> = contents.clone();
                    let bindings: Vec<(String, Value)> = bind_names.into_iter().zip(bind_values).collect();

                    let extended_ctx = ctx.extend_many(&bindings);
                    eval(body, extended_ctx, runtime)
                },
                _ => panic!(format!("Expected a constructor during match statement, but found {:?}", &t_value)),
            }
        },
        TermNode::Hole(hole_info) => hole::fill(runtime, hole_info, ctx),
    }
}

