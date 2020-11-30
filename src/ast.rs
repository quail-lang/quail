use std::rc;

#[derive(Clone, Debug)]
pub enum Type {
    Bool,
    Nat,
    Arrow(Box<Type>, Box<Type>),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum PrimFn {
    Succ,
    Add,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Term(pub rc::Rc<TermNode>);

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermNode {
    Var(String),
    Lam(String, Term),
    App(Term, Term),
    BoolLit(bool),
    NatLit(u64),
    PrimApp(PrimFn, Vec<Term>),
}
