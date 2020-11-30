use crate::ast::Term;
use crate::ast::PrimFn;
use crate::ast::TermNode::*;

pub fn eval(t: Term) -> Term {
    use crate::ast::TermNode::*;
    match t.as_ref() {
        Var(_x) => t.clone(),
        Lam(x, ty, body) => Lam(x.clone(), ty.clone(), eval(body.clone())).into(),
        App(f, v) => match f.as_ref() {
            Lam(x, _ty, body) => {
                let reduced_body = subst(body.clone(), x.clone(), v.clone());
                eval(reduced_body)
            },
            _ => panic!("Applied argument to non-function."),
        },
        BoolLit(_b) => t.clone(),
        NatLit(_n) => t.clone(),
        PrimApp(prim_fn, vs) => eval_prim(prim_fn.clone(), vs.clone()),
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

pub fn subst(t: Term, x: String, v: Term) -> Term {
    match t.as_ref() {
        Var(y) => {
            if x == *y {
                v.clone()
            } else {
                t.clone()
            }
        }
        Lam(y, ty, body) => {
            if x == *y {
                t.clone()
            } else {
                Lam(y.to_string(), ty.clone(), subst(body.clone(), x, v)).into()
            }
        }
        App(f, w) => App(
            subst(f.clone(), x.clone(), v.clone()),
            subst(w.clone(), x.clone(), v.clone()),
        ).into(),
        BoolLit(_b) => t.clone(),
        NatLit(_n) => t.clone(),
        PrimApp(_f, _vs) => t.clone()
    }
}
