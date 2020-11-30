use std::rc;

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
    match v {
        Value::Nat(n) => Value::Nat(n + 1),
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
        .extend(&"succ".into(), Value::Prim(rc::Rc::new(Box::new(succ_prim))))
        .extend(&"pred".into(), Value::Prim(rc::Rc::new(Box::new(pred_prim))))
        .extend(&"println".into(), Value::Prim(rc::Rc::new(Box::new(println_prim))))
        .extend(&"ifzero".into(), Value::Prim(rc::Rc::new(Box::new(ifzero_prim))))
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
        NatLit(n) => Value::Nat(*n),
    }
}
