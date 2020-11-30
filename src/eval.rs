use std::rc;

use crate::parser;

use crate::ast;
use crate::ast::Program;
use crate::ast::Term;
use crate::ast::Item;

use crate::ast::Value;
use crate::ast::Context;

pub fn exec(program: &Program) {
    let Item::Def(_, main_body) = program.def("main").expect("There should be a main in your program").clone();
    eval(main_body, prelude_ctx(), program);
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

fn apply(func: Value, args: Vec<Value>, program: &Program) -> Value {
    match &func {
        Value::Fun(x, body, local_ctx) => {
            match args.clone().split_first() {
                None => func,
                Some((v, vs_remaining)) => {
                    let new_ctx = local_ctx.extend(&x, v.clone());
                    let new_func = eval(body.clone(), new_ctx, program);
                    apply(new_func, vs_remaining.to_vec(), program)
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

pub fn eval(t: Term, ctx: Context, program: &Program) -> Value {
    use crate::ast::TermNode::*;
    match t.as_ref() {
        Var(x) => {
            match ctx.lookup(x) {
                Some(v) => v,
                None => {
                    let Item::Def(_, body) = program.def(x.to_string()).expect(&format!("Unbound variable {:?}", &x));
                    eval(body.clone(), ctx, program)
                },
            }
        },
        Lam(x, body) => {
            Value::Fun(x.clone(), body.clone(), ctx.clone())
        },
        App(f, vs) => {
            let f_value = eval(f.clone(), ctx.clone(), program);
            let vs_values: Vec<Value> = vs.iter().map(|v| eval(v.clone(), ctx.clone(), program)).collect();
            apply(f_value, vs_values, program)
        },
        Let(x, v, body) => {
            let v_value = eval(v.clone(), ctx.clone(), program);
            let extended_ctx = ctx.extend(x, v_value);
            eval(body.clone(), extended_ctx, program)
        },
        Match(t, match_arms) => {
            let t_value = eval(t.clone(), ctx.clone(), program);
            match t_value {
                Value::Ctor(tag, contents) => {
                    let ast::MatchArm(pat, body) = ast::find_matching_arm(&tag, &match_arms);

                    let bind_names: Vec<String> = pat[1..].into_iter().map(|name| name.clone()).collect();
                    let bind_values: Vec<Value> = contents.clone();
                    let bindings: Vec<(String, Value)> = bind_names.into_iter().zip(bind_values).collect();

                    let extended_ctx = ctx.extend_many(&bindings);
                    eval(body, extended_ctx, program)
                },
                _ => panic!(format!("Expected a constructor during match statement, but found {:?}", &t_value)),
            }
        },
        Hole(contents) => eval_hole(ctx, program, contents),
    }
}

fn eval_hole(ctx: Context, program: &Program, contents: &str) -> Value {
    println!("Encountered hole:");
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
    for item in program.items.iter() {
        let Item::Def(name, _) = item;
        println!("        {}", &name);
    }

    println!("");
    interp(ctx, program)
}

fn interp(ctx: Context, program: &Program) -> Value {
    let mut program_text = String::new();
    print!("> ");
    use std::io::Write;
    std::io::stdout().flush().expect("Couldn't flush stdout??");
    std::io::stdin().read_line(&mut program_text).expect("Couldn't read from stdin??");

    match parser::parse_term(program_text) {
        Ok(term) => eval(term, ctx, program),
        Err(e) => panic!(format!("There was an error {:?}", e)),
    }
}
