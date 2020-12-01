use std::rc;

use crate::parser;
use crate::tokenizer::Loc;

#[derive(Clone, Debug)]
pub struct Module {
    pub definitions: Vec<Def>,
    pub imports: Vec<Import>,
}

#[derive(Clone, Debug)]
pub struct Def(pub String, pub Type, pub Term);

#[derive(Clone, Debug)]
pub struct Import(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Term(Box<TermNode>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub layer: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermNode {
    Var(Variable),
    Lam(String, Term),
    App(Term, Vec<Term>),
    Let(String, Term, Term),
    Match(Term, Vec<MatchArm>),
    Hole(HoleInfo),
    As(Term, Type),
    StrLit(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoleInfo {
    pub hole_id: HoleId,
    pub name: Option<String>,
    pub contents: Option<String>,
    pub loc: Loc,
}

pub type HoleId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchArm(pub Pattern, pub Term);

pub type Pattern = Vec<String>;

pub type Tag = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type(rc::Rc<TypeNode>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeNode {
    Atom(String),
    Arrow(Type, Type),
    Forall(String, Type),
}

impl std::ops::Deref for Term {
    type Target = TermNode;

    fn deref(&self) -> &TermNode {
        use std::borrow::Borrow;
        let Term(rc_tn) = self;
        rc_tn.borrow()
    }
}

impl AsRef<TermNode> for Term {
    fn as_ref(&self) -> &TermNode {
        use std::borrow::Borrow;
        let Term(rc_tn) = self;
        rc_tn.borrow()
    }
}

impl From<TermNode> for Term {
    fn from(tn: TermNode) -> Self {
        Term(Box::new(tn))
    }
}

impl Module {
    pub fn new(definitions: Vec<Def>, imports: Vec<Import>) -> Self {
        Module { definitions, imports }
    }
}


impl HoleInfo {
    pub fn new(hole_id: HoleId, name: Option<String>, contents: Option<String>, loc: Loc) -> Self {
        HoleInfo {
            hole_id,
            name,
            contents,
            loc,
        }
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.layer == 0 {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}${}", self.name, self.layer)
        }
    }

}

pub fn find_matching_arm(tag: &Tag, match_arms: &[MatchArm]) -> MatchArm {
    for match_arm in match_arms {
        let MatchArm(pat, _body) = match_arm;
        if pat[0] == *tag {
            return match_arm.clone();
        }
    }
    panic!(format!("No matching arm found for tag {:?}", tag))
}

impl From<TypeNode> for Type {
    fn from(tn: TypeNode) -> Self {
        Type(rc::Rc::new(tn))
    }
}

impl std::convert::TryFrom<&str> for Type {
    type Error = parser::ParseErr;

    fn try_from(typ: &str) -> Result<Self, Self::Error> {
        parser::parse_type(None, typ)
    }
}

impl std::ops::Deref for Type {
    type Target = TypeNode;

    fn deref(&self) -> &TypeNode {
        use std::borrow::Borrow;
        let Type(rc_tn) = self;
        rc_tn.borrow()
    }
}

impl AsRef<TypeNode> for Type {
    fn as_ref(&self) -> &TypeNode {
         use std::borrow::Borrow;
         let Type(rc_tn) = self;
         rc_tn.borrow()
    }
}

impl std::fmt::Display for TypeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypeNode::Atom(atom) => write!(f, "{}", atom),
            TypeNode::Arrow(dom, cod) => {
                if let TypeNode::Atom(_) = **dom {
                    write!(f, "{}", **dom)?;
                } else {
                    write!(f, "({})",**dom)?;
                }
                write!(f, " -> ")?;
                write!(f, "{}", **cod)
            }
            TypeNode::Forall(atom, typ) => write!(f, "[{}] -> {}", atom, **typ),
        }
    }
}
