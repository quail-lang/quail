use std::collections;
use std::rc;

#[derive(Clone, Debug)]
enum Type {
    Bool,
    Nat,
    Arrow(Box<Type>, Box<Type>),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum PrimFn {
    Succ,
    Add,
}

#[derive(Clone, Debug)]
struct Term(rc::Rc<TermNode>);

impl AsRef<TermNode> for Term {
    fn as_ref(&self) -> &TermNode {
        use std::borrow::Borrow;
        let Term(rc_tn) = self;
        rc_tn.borrow()
    }
}

impl From<TermNode> for Term {
    fn from(tn: TermNode) -> Self {
        Term(rc::Rc::new(tn))
    }
}

#[derive(Clone, Debug)]
enum TermNode {
    Var(String),
    Lam(String, Type, Term),
    App(Term, Term),
    BoolLit(bool),
    NatLit(u64),
    PrimApp(PrimFn, Vec<Term>),
}

fn eval(t: Term) -> Term {
    use TermNode::*;
    match t.as_ref() {
        Var(x) => t.clone(),
        Lam(x, ty, body) => Lam(x.clone(), ty.clone(), eval(body.clone())).into(),
        App(f, v) => match f.as_ref() {
            Lam(x, _ty, body) => {
                let reduced_body = subst(body.clone(), x.clone(), v.clone());
                eval(reduced_body)
            },
            _ => panic!("Applied argument to non-function."),
        },
        BoolLit(b) => t.clone(),
        NatLit(n) => t.clone(),
        PrimApp(prim_fn, vs) => eval_prim(prim_fn.clone(), vs.clone()),
    }
}

fn eval_prim(prim_fn: PrimFn, vs: Vec<Term>) -> Term {
    use TermNode::*;
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

fn subst(t: Term, x: String, v: Term) -> Term {
    use TermNode::*;
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
        BoolLit(b) => t.clone(),
        NatLit(n) => t.clone(),
        PrimApp(f, vs) => t.clone()
    }
}

fn type_of(t: Term) -> Type {
    let ctx = collections::HashMap::new();
    type_of_ctx(t, ctx)
}

fn type_of_ctx(t: Term, ctx: collections::HashMap<String, Type>) -> Type {
    use TermNode::*;
    match t.as_ref() {
        Var(y) => {
            ctx.get(y).unwrap().clone()
        }
        Lam(y, ty, body) => {
            // TODO
            let mut new_ctx = ctx.clone();
            new_ctx.insert(y.clone(), ty.clone());
            let cod = type_of_ctx(body.clone(), new_ctx);
            Type::Arrow(Box::new(ty.clone()), Box::new(cod))
        }
        App(f, w) => {
            // TODO
            let f_ty = type_of_ctx(f.clone(), ctx.clone());
            let w_ty = type_of_ctx(w.clone(), ctx.clone());
            if let Type::Arrow(dom, cod) = f_ty {
                *cod
            } else {
                panic!("Impossible"); // TODO
            }
        },
        BoolLit(b) => Type::Bool,
        NatLit(n) => Type::Nat,
        PrimApp(f, vs) => type_of_prim(*f, vs.clone()),
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

fn main() {
    use TermNode::*;
    let term: Term = PrimApp(
        PrimFn::Add,
        vec![Term(rc::Rc::new(NatLit(5))), Term(rc::Rc::new(NatLit(5)))],
    ).into();


    let term2: Term = App(
        Lam("x".to_string(), Type::Nat, Var("x".to_string()).into()).into(),
        term.clone(),
    ).into();
    dbg!(&term2);
    dbg!(eval(term2.clone()));
    dbg!(type_of(term2.clone()));
    dbg!(type_of(eval(term2.clone())));
}
