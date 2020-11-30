use std::rc;

use crate::ast::Term;
use crate::ast::PrimFn;
use crate::ast::TermNode::*;

use crate::ast::Value;
use crate::ast::Context;

pub fn eval(t: Term) -> Value {
    eval_ctx(t, Context::empty())
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
                _ => panic!("Can't apply a value to a non-function {:?}.", &f),
            }
        },
        NatLit(n) => Value::Nat(*n),
        PrimApp(prim_fn, vs) => unimplemented!(), //eval_prim(prim_fn.clone(), vs.clone()),
    }
}

fn eval_prim(prim_fn: PrimFn, vs: Vec<Term>) -> Term {
    match prim_fn {
        PrimFn::Succ => {
            assert!(vs.len() == 1, "Succ takes 1 argument.");
            let v = &vs[0];
            if let NatLit(n) = v.as_ref() {
                NatLit(n + 1).into()
            } else {
                panic!("Can't succ on non-Nat.")
            }
        },
        PrimFn::Add => {
            assert!(vs.len() == 2, "Add takes 2 arguments.");
            let v1 = &vs[0];
            let v2 = &vs[1];
            if let NatLit(n) = v1.as_ref() {
                if let NatLit(m) = v2.as_ref() {
                    return NatLit(n + m).into()
                }
            }
            panic!("Can't add on non-Nat.")
        },
    }
}
