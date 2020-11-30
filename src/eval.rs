use std::rc;

use crate::ast;
use crate::ast::Program;
use crate::ast::Term;
use crate::ast::TermNode;
use crate::ast::Item;

use crate::ast::Value;
use crate::ast::Context;

pub fn exec(program: &Program) {
    let Item::Def(_, main_body) = program.item("main").expect("There should be a main in your program").clone();
    eval(main_body, prelude_ctx(), program);
}

fn succ_prim(v: Value) -> Value {
    match v.clone() {
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

fn println_prim(v: Value) -> Value {
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

}

pub fn eval(t: Term, ctx: Context, program: &Program) -> Value {
    use crate::ast::TermNode::*;
    match t.as_ref() {
        Var(x) => {
            match ctx.lookup(x) {
                Some(v) => v,
                None => {
                    let Item::Def(_, body) = program.item(x.to_string()).expect(&format!("Unbound variable {:?}", &x));
                    eval(body.clone(), ctx, program)
                },
            }
        },
        Lam(x, body) => {
            Value::Fun(x.clone(), body.clone(), ctx.clone())
        },
        App(f, v) => {
            match eval(f.clone(), ctx.clone(), program) {
                Value::Fun(x, body, local_ctx) => {
                    let v_value = eval(v.clone(), ctx.clone(), program);
                    eval(body, local_ctx.extend(&x, v_value), program)
                },
                Value::Prim(prim) => {
                    let v_value = eval(v.clone(), ctx.clone(), program);
                    prim(v_value)
                },
                _ => panic!("Can't apply a value to a non-function {:?}.", &f),
            }
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
        Hole => eval_hole(ctx, program),
    }
}

fn eval_hole(ctx: Context, program: &Program) -> ! {
    println!("Context: {:?}", ctx);
    println!("Program: {:?}", program);
    panic!("Encountered hole");
}
