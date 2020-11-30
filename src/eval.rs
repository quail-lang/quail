use std::collections::HashMap;
use std::rc;

use crate::parser;

use crate::ast;
use crate::ast::Term;

use crate::ast::Def;
use crate::ast::Value;
use crate::ast::Context;
use ast::HoleId;

#[derive(Clone, Debug)]
pub struct Runtime {
    pub imports: Vec<String>,
    pub definitions: Vec<ast::Def>,
    pub holes: HashMap<HoleId, Value>,
}

impl Runtime {
    pub fn load(filepath: impl AsRef<std::path::Path>) -> Self {
        let mut runtime = Runtime {
            imports: vec![],
            definitions: vec![],
            holes: HashMap::new(),
        };

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
        fs::File::open(filepath)
            .unwrap_or_else(|e| panic!(format!("There was an error {:?}", e)))
            .read_to_string(&mut module_text)
            .unwrap_or_else(|e| panic!(format!("There was an error {:?}", e)));

        let module = parser::parse_module(module_text)
            .unwrap_or_else(|e| panic!(format!("There was an error {:?}", e)));

        if is_main {
            self.definitions.extend(module.definitions);
        } else {
            self.definitions.extend(module.definitions.into_iter().filter(|Def(name, _)| name != "main"));
        }
        for import in module.imports {
            let ast::Import(name) = import;

            let mut import_filename = name.to_string();
            import_filename.extend(".ql".chars());
            let import_filename = &std::path::Path::new(&import_filename);

            self.load_module(import_filename, basedir, false);
        }
    }

    fn definition(&self, name: impl Into<String>) -> Option<&Def> {
        let name: String = name.into();
        for definition in &self.definitions {
            let Def(def_name, _) = &definition;
            if *def_name == name {
                return Some(definition);
            }
        }
        None
    }

    pub fn exec(&mut self) {
        let ast::Def(_, main_body) = self.definition("main").expect("There should be a main in your module").clone();
        eval(main_body, prelude_ctx(), self);
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
    assert_eq!(vs.len(), 1, "succ must have exactly one argument");
    let v = vs[0].clone();
    println!("{:?}", v);
    v
}

pub fn prelude_ctx() -> Context {
    Context::empty()
        .extend(&"println".into(), Value::Prim(rc::Rc::new(Box::new(println_prim))))
        .extend(&"zero".into(), Value::Ctor("zero".into(), Vec::new()))
        .extend(&"succ".into(), Value::Prim(rc::Rc::new(Box::new(succ_prim))))
        .extend(&"true".into(), Value::Ctor("true".into(), Vec::new()))
        .extend(&"false".into(), Value::Ctor("false".into(), Vec::new()))
        .extend(&"nil".into(), Value::Ctor("nil".into(), Vec::new()))
        .extend(&"cons".into(), Value::Prim(rc::Rc::new(Box::new(cons_prim))))
        .extend(&"unit".into(), Value::Ctor("unit".into(), Vec::new()))
        .extend(&"pair".into(), Value::Prim(rc::Rc::new(Box::new(pair_prim))))
}

fn apply(func: Value, args: Vec<Value>, runtime: &Runtime) -> Value {
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
    }
}

pub fn eval(t: Term, ctx: Context, runtime: &Runtime) -> Value {
    use crate::ast::TermNode::*;
    match t.as_ref() {
        Var(x) => {
            match ctx.lookup(x) {
                Some(v) => v,
                None => {
                    let ast::Def(_, body) = runtime.definition(x.to_string()).expect(&format!("Unbound variable {:?}", &x));
                    eval(body.clone(), ctx, runtime)
                },
            }
        },
        Lam(x, body) => {
            Value::Fun(x.clone(), body.clone(), ctx.clone())
        },
        App(f, vs) => {
            let f_value = eval(f.clone(), ctx.clone(), runtime);
            let vs_values: Vec<Value> = vs.iter().map(|v| eval(v.clone(), ctx.clone(), runtime)).collect();
            apply(f_value, vs_values, runtime)
        },
        Let(x, v, body) => {
            let v_value = eval(v.clone(), ctx.clone(), runtime);
            let extended_ctx = ctx.extend(x, v_value);
            eval(body.clone(), extended_ctx, runtime)
        },
        Match(t, match_arms) => {
            let t_value = eval(t.clone(), ctx.clone(), runtime);
            match t_value {
                Value::Ctor(tag, contents) => {
                    let ast::MatchArm(pat, body) = ast::find_matching_arm(&tag, &match_arms);

                    let bind_names: Vec<String> = pat[1..].into_iter().map(|name| name.clone()).collect();
                    let bind_values: Vec<Value> = contents.clone();
                    let bindings: Vec<(String, Value)> = bind_names.into_iter().zip(bind_values).collect();

                    let extended_ctx = ctx.extend_many(&bindings);
                    eval(body, extended_ctx, runtime)
                },
                _ => panic!(format!("Expected a constructor during match statement, but found {:?}", &t_value)),
            }
        },
        Hole(hole_id, contents) => eval_hole(*hole_id, ctx, runtime, contents),
    }
}

fn eval_hole(hole_id: HoleId, ctx: Context, runtime: &Runtime, contents: &str) -> Value {
    println!("Encountered hole #{}", hole_id);
    println!("");
    if contents != "" {
        println!("    Note: {:?}", contents);
    }

    println!("");
    println!("    Bindings:");
    for (name, value) in ctx.bindings().into_iter() {
        println!("        {} = {:?}", name, &value);
    }

    println!("");
    println!("    Globals:");
    for definition in runtime.definitions.iter() {
        let ast::Def(name, _) = definition;
        println!("        {}", &name);
    }

    println!("");

    let mut term_text = String::new();
    print!("> ");
    use std::io::Write;
    std::io::stdout().flush().expect("Couldn't flush stdout??");
    std::io::stdin().read_line(&mut term_text).expect("Couldn't read from stdin??");

    match parser::parse_term(term_text) {
        Ok(term) => eval(term, ctx, runtime),
        Err(e) => panic!(format!("There was an error {:?}", e)),
    }
}
