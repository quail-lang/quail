use std::rc;

use crate::ast::Term;
use crate::ast::PrimFn;
use crate::ast::TermNode::*;

use crate::ast::Value;
use crate::ast::Context;

fn succ_prim(v: Value) -> Value {
    match v {
        Value::Nat(n) => Value::Nat(n + 1),
        other => panic!(format!("Couldn't succ {:?}", other)),
    }
}

pub fn eval(t: Term) -> Value {
    //eval_ctx(t, Context::empty())

    let ctx = Context::empty()
        .extend(&"succ".into(), Value::Prim(rc::Rc::new(Box::new(succ_prim))));

    eval_ctx(t, ctx)
}

pub fn eval_ctx(t: Term, ctx: Context) -> Value {
    use crate::ast::TermNode::*;
    match t.as_ref() {
        Var(x) => ctx.lookup(x).expect(&format!("I wanted a value for {:?} in the context {:?}, lol!", x, ctx)),
        Lam(x, body) => {
            Value::Fun(x.clone(), body.clone(), ctx.clone())
        },
        App(f, v) => {
            match eval_ctx(f.clone(), ctx.clone()) {
                Value::Fun(x, body, local_ctx) => {
                    let v_value = eval_ctx(v.clone(), ctx.clone());
                    eval_ctx(body, local_ctx.extend(&x, v_value))
                },
                Value::Prim(prim) => {
                    let v_value = eval_ctx(v.clone(), ctx.clone());
                    prim(v_value)
                },
                _ => panic!("Can't apply a value to a non-function {:?}.", &f),
            }
        },
        NatLit(n) => Value::Nat(*n),
    }
}
