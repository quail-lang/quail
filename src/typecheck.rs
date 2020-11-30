use super::ast;
use super::ast::Type;
use super::ast::Term;
use super::ast::PrimFn;

use std::collections::HashMap;

pub fn type_of(t: Term) -> Type {
    let ctx = HashMap::new();
    type_of_ctx(t, ctx)
}

fn type_of_ctx(t: Term, ctx: HashMap<String, Type>) -> Type {
    match t.as_ref() {
        ast::TermNode::Var(y) => {
            ctx.get(y).unwrap().clone()
        }
        ast::TermNode::Lam(y, ty, body) => {
            // TODO
            let mut new_ctx = ctx.clone();
            new_ctx.insert(y.clone(), ty.clone());
            let cod = type_of_ctx(body.clone(), new_ctx);
            ast::Type::Arrow(Box::new(ty.clone()), Box::new(cod))
        }
        ast::TermNode::App(f, w) => {
            // TODO
            let f_ty = type_of_ctx(f.clone(), ctx.clone());
            let _w_ty = type_of_ctx(w.clone(), ctx.clone());
            if let ast::Type::Arrow(_dom, cod) = f_ty {
                *cod
            } else {
                panic!("Impossible"); // TODO
            }
        },
        ast::TermNode::BoolLit(_b) => Type::Bool,
        ast::TermNode::NatLit(_n) => Type::Nat,
        ast::TermNode::PrimApp(f, vs) => type_of_prim(*f, vs.clone()),
    }
}

fn type_of_prim(prim_fn: PrimFn, vs: Vec<Term>) -> Type {
    match prim_fn {
        PrimFn::Succ => {
            assert!(vs.len() == 1, "Succ takes 1 argument.");
            let _v = &vs[0];
            Type::Nat
        },
        PrimFn::Add => {
            assert!(vs.len() == 2, "Add takes 2 argument.");
            let _v1 = &vs[0];
            let _v2 = &vs[1];
            Type::Nat
        }
    }
}
