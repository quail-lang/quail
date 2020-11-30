use std::rc;
use std::fmt;

pub struct Program {
    pub items: Vec<Item>,
}

impl Program {
    pub fn item(&self, name: impl Into<String>) -> Option<&Item> {
        let name: String = name.into();
        for item in &self.items {
            let Item::Def(item_name, _) = &item;
            if *item_name == name {
                return Some(item);
            }
        }
        None
    }
}

pub enum Item {
    Def(String, Term),
}

#[derive(Clone, Debug)]
pub enum Type {
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
    Let(String, Term, Term),
    NatLit(u64),
}

#[derive(Clone)]
pub enum Value {
    Nat(u64),
    Fun(String, Term, Context),
    Prim(rc::Rc<Box<Fn(Value) -> Value>>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nat(n) => write!(f, "{}", n),
            Value::Fun(_, _, _) => write!(f, "<fun>"),
            Value::Prim(_) => write!(f, "<prim>"),
        }
    }
}

#[derive(Debug)]
struct ContextNode(Vec<(String, Value)>);

#[derive(Debug, Clone)]
pub struct Context(rc::Rc<ContextNode>);

impl Context {
    pub fn empty() -> Self {
        Context(rc::Rc::new(ContextNode(Vec::new())))
    }

    pub fn lookup(&self, x: &String) -> Option<Value> {
        let Context(rc_ctx_node) = self;
        let ContextNode(var_val_list) = rc_ctx_node.as_ref();
        for (y, value) in var_val_list.iter().rev() {
            if x == y {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn extend(&self, x: &String, v: Value) -> Context {
        let Context(rc_ctx_node) = self;
        let ContextNode(var_val_list) = rc_ctx_node.as_ref();
        let mut extended_var_val_list = var_val_list.clone();
        extended_var_val_list.push((x.to_string(), v.clone()));
        Context(rc::Rc::new(ContextNode(extended_var_val_list)))
    }
}
