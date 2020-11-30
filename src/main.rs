use std::collections;
use std::fmt;
use std::rc;

#[derive(Clone, Debug)]
enum Type {
    Bool,
    Nat,
    Arrow(Box<Type>, Box<Type>),
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum PrimFn {
    Succ,
    Add,
}

impl fmt::Debug for PrimFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<primitive function>")
    }
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

struct Context(collections::HashMap<String, Term>);

impl Context {
    fn empty() -> Self {
        Context(collections::HashMap::new())
    }
}

fn eval(t: Term, _ctx: &Context) -> Term {
    use TermNode::*;
    match t.as_ref() {
        Var(x) => Var(x.clone()).into(),
        Lam(x, ty, body) => Lam(x.clone(), ty.clone(), body.clone()).into(),
        App(f, v) => return match f.as_ref() {
            Lam(x, _ty, body) => subst(body.clone(), x.clone(), v.clone()).into(),
            _ => panic!("Applied argument to non-function."),
        },
        BoolLit(b) => BoolLit(*b).into(),
        NatLit(n) => NatLit(*n).into(),
        PrimApp(prim_fn, vs) => eval_prim(prim_fn.clone(), vs.clone()),
    }
}

fn eval_prim(prim_fn: PrimFn, vs: Vec<Term>) -> Term {
    use TermNode::*;
    use PrimFn::*;
    match prim_fn {
        Succ => {
            assert!(vs.len() == 1, "Succ takes 1 argument.");
            let v = &vs[0];
            if let NatLit(n) = v.as_ref() {
                NatLit(n + 1).into()
            } else {
                panic!("Can't succ on non-Nat.")
            }
        },
        Add => {
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
                Var(y.clone()).into()
            }
        }
        Lam(y, ty, body) => {
            if x == *y {
                Lam(x.clone(), ty.clone(), body.clone()).into()
            } else {
                Lam(y.to_string(), ty.clone(), subst(body.clone(), x, v)).into()
            }
        }
        App(f, w) => App(
            subst(f.clone(), x.clone(), v.clone()),
            subst(w.clone(), x.clone(), v.clone()),
        ).into(),
        BoolLit(b) => BoolLit(*b).into(),
        NatLit(n) => NatLit(*n).into(),
        PrimApp(f, vs) => PrimApp(f.clone(), vs.clone()).into(),
    }
}

fn main() {
    use TermNode::*;
    let term = Term(rc::Rc::new(PrimApp(
        PrimFn::Add,
        vec![Term(rc::Rc::new(NatLit(5))), Term(rc::Rc::new(NatLit(5)))],
    )));
    dbg!(eval(term, &Context::empty()));
}
