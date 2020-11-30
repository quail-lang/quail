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

#[derive(Clone, Debug)]
enum TermNode {
    Var(String),
    Lam(String, Type, Term),
    App(Term, Term),
    BoolLit(bool),
    NatLit(u64),
    Prim(PrimFn),
}

struct Context(collections::HashMap<String, Term>);

impl Context {
    fn empty() -> Self {
        Context(collections::HashMap::new())
    }
}

fn eval(t: Term, _ctx: &Context) -> Term {
    use TermNode::*;
    let s = match t.as_ref() {
        Var(x) => Var(x.clone()),
        Lam(x, ty, body) => Lam(x.clone(), ty.clone(), body.clone()),
        App(f, v) => return match f.as_ref() {
            Lam(x, ty, body) => subst(body.clone(), x.clone(), v.clone()),
            Prim(prim_fn) => eval_prim(*prim_fn, v.clone()),
            _ => panic!("Applied argument to non-function."),
        },
        BoolLit(b) => BoolLit(*b),
        NatLit(n) => NatLit(*n),
        Prim(prim_fn) => Prim(prim_fn.clone()),
    };
    Term(rc::Rc::new(s))
}

fn eval_prim(prim_fn: PrimFn, v: Term) -> Term {
    use TermNode::*;
    match prim_fn {
        Succ => {
            if let NatLit(n) = v.as_ref() {
                Term(rc::Rc::new(NatLit(n + 1)))
            } else {
                panic!("Can't succ on non-Nat.")
            }
        }
    }
}

fn subst(t: Term, x: String, v: Term) -> Term {
    use TermNode::*;
    let s = match t.as_ref() {
        Var(y) => {
            if x == *y {
                return v.clone()
            } else {
                Var(y.clone())
            }
        }
        Lam(y, ty, body) => {
            if x == *y {
                Lam(x.clone(), ty.clone(), body.clone())
            } else {
                Lam(y.to_string(), ty.clone(), subst(body.clone(), x, v))
            }
        }
        App(f, w) => App(
            subst(f.clone(), x.clone(), v.clone()),
            subst(w.clone(), x.clone(), v.clone()),
        ),
        BoolLit(b) => BoolLit(*b),
        NatLit(n) => NatLit(*n),
        Prim(f) => Prim(f.clone()),
    };

    Term(rc::Rc::new(s))
}

fn main() {
    use TermNode::*;
    let term = Term(rc::Rc::new(App(
        Term(rc::Rc::new(Prim(PrimFn::Succ))),
        Term(rc::Rc::new(NatLit(5)))))
    );
    dbg!(eval(term, &Context::empty()));
}
