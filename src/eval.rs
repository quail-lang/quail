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
        Value::Nat(n) => Value::Nat(n + 1),
        Value::Ctor(tag, _) => {
            if tag == "Zero" {
                Value::Ctor("Succ".into(), vec![Value::Ctor("Zero".into(), vec![])])
            } else if tag == "Succ" {
                Value::Ctor("Succ".into(), vec![v.clone()])
            } else {
                panic!("Invalid thing to succ: {:?}", &v)
            }
        },
        other => panic!(format!("Couldn't succ {:?}", other)),
    }
}

fn pred_prim(v: Value) -> Value {
    match v {
        Value::Nat(0) => Value::Nat(0),
        Value::Nat(n) => Value::Nat(n - 1),
        other => panic!(format!("Couldn't pred {:?}", other)),
    }
}

fn println_prim(v: Value) -> Value {
    println!("{:?}", v);
    Value::Nat(0)
}

fn ifzero_prim(v: Value) -> Value {
    if let Value::Nat(n) = v {
        if n == 0 {
            Value::Fun(
                "x".to_string(),
               TermNode::Lam("y".into(), TermNode::Var("x".into()).into()).into(),
               Context::empty(),
            )
        } else {
            Value::Fun(
                "x".to_string(),
                TermNode::Lam("y".into(), TermNode::Var("y".into()).into()).into(),
                Context::empty(),
            )
        }
    } else {
        panic!(format!("Expected a number, but got {:?}", v));
    }
}

pub fn prelude_ctx() -> Context {
    Context::empty()
        .extend(&"pred".into(), Value::Prim(rc::Rc::new(Box::new(pred_prim))))
        .extend(&"println".into(), Value::Prim(rc::Rc::new(Box::new(println_prim))))
        .extend(&"ifzero".into(), Value::Prim(rc::Rc::new(Box::new(ifzero_prim))))
        .extend(&"Zero".into(), Value::Ctor("Zero".into(), Vec::new()))
        .extend(&"Succ".into(), Value::Prim(rc::Rc::new(Box::new(succ_prim))))
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
        NatLit(n) => Value::Nat(*n),
    }
}
