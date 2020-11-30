use std::rc;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Module {
    pub definitions: Vec<Def>,
    pub imports: Vec<Import>,
}

impl Module {
    pub fn new(definitions: Vec<Def>, imports: Vec<Import>) -> Self {
        Module { definitions, imports }
    }
}

#[derive(Clone, Debug)]
pub struct Def(pub String, pub Term);

#[derive(Clone, Debug)]
pub struct Import(pub String);

#[derive(Clone, Debug)]
pub enum Type {
    Nat,
    Arrow(Box<Type>, Box<Type>),
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
    App(Term, Vec<Term>),
    Let(String, Term, Term),
    Match(Term, Vec<MatchArm>),
    Hole(HoleInfo),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoleInfo {
    pub hole_id: HoleId,
    pub name: Option<String>,
    pub contents: Option<String>,
}

impl HoleInfo {
    pub fn new(hole_id: HoleId, name: Option<String>, contents: Option<String>) -> Self {
        HoleInfo {
            hole_id,
            name,
            contents,
        }
    }
}

pub type HoleId = usize;

pub fn find_matching_arm(tag: &CtorTag, match_arms: &Vec<MatchArm>) -> MatchArm {
    for match_arm in match_arms {
        let MatchArm(pat, _body) = match_arm;
        if pat[0] == *tag {
            return match_arm.clone();
        }
    }
    panic!(format!("No matching arm found for tag {:?}", tag))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchArm(pub Pattern, pub Term);

pub type Pattern = Vec<String>;

#[derive(Clone)]
pub enum Value {
    Ctor(CtorTag, Vec<Value>),
    Fun(String, Term, Context),
    Prim(rc::Rc<Box<Fn(Vec<Value>) -> Value>>),
    Str(String),
}

pub type CtorTag = String;

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Ctor(tag, contents) => {
                write!(f, "{}", &tag)?;
                for value in contents {
                    write!(f, " ({:?})", value)?;
                }
                Ok(())
            },
            Value::Str(s) => write!(f, "{}", s),
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

    pub fn extend_many(&self, bindings: &[(String, Value)]) -> Context {
        let mut ctx = self.clone();
        for (name, value) in bindings.iter() {
            ctx = ctx.extend(name, value.clone());
        }
        ctx
    }

    pub fn bindings(&self) -> Vec<(String, Value)> {
        let Context(context_node_rc) = self;
        let ContextNode(bindings) = context_node_rc.as_ref();
        bindings.clone()
    }
}
