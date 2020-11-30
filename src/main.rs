use std::fmt;
use std::rc;

#[derive(Clone, Debug)]
enum Type {
    Bool,
    Nat,
    Arrow(Box<Type>, Box<Type>),
}

#[derive(Clone)]
struct PrimFn(rc::Rc<Fn(&Term) -> Term>);

impl fmt::Debug for PrimFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<primitive function>")
    }
}

#[derive(Clone, Debug)]
enum Term {
    Var(String),
    Lam(String, Type, Box<Term>),
    App(Box<Term>, Box<Term>),
    BoolLit(bool),
    NatLit(u64),
    Prim(PrimFn),
}

fn eval(t: &Term) -> Term {
    use Term::*;
    match t {
        Var(x) => Var(x.clone()),
        Lam(x, ty, body) => Lam(x.clone(), ty.clone(), body.clone()),
        App(f, v) => match *f.clone() {
            Lam(x, ty, body) => subst(&body, x.clone(), &v.clone()),
            Prim(PrimFn(prim_f)) => (*prim_f)(&v.clone()),
            _ => panic!("Applied argument to non-function."),
        },
        BoolLit(b) => BoolLit(*b),
        NatLit(n) => NatLit(*n),
        Prim(f) => Prim(f.clone()),
    }
}

fn subst(t: &Term, x: String, v: &Term) -> Term {
    use Term::*;
    match t {
        Var(y) => {
            if x == *y {
                v.clone()
            } else {
                Var(y.clone())
            }
        },
        Lam(y, ty, body) => {
            if x == *y {
                Lam(x.clone(), ty.clone(), body.clone())
            } else {
                Lam(y.to_string(), ty.clone(), Box::new(subst(body, x, v)))
            }
        },
        App(f, w) => App(Box::new(subst(f, x.clone(), v)), Box::new(subst(w, x.clone(), v))),
        BoolLit(b) => BoolLit(*b),
        NatLit(n) => NatLit(*n),
        Prim(f) => Prim(f.clone()),
    }
}

fn main() {
    use Term::*;
    fn addOneFn(t: &Term) -> Term {
        match t {
            NatLit(n) => NatLit(n + 1),
            _ => panic!("No idea how to add 1 to this!"),
        }
    }
    let term = App(Box::new(Prim(PrimFn(rc::Rc::new(addOneFn)))), Box::new(NatLit(5)));
    dbg!(eval(&term));
}
