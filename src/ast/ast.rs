use std::collections::HashSet;
use std::rc;

use crate::parser;
use crate::tokenizer::Loc;

use std::convert::TryFrom;

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

    pub fn definition(&self, name: &str) -> Option<Def> {
        for d in &self.definitions {
            let Def(definition_name, _typ, _body) = d;
            if definition_name == name {
                return Some(d.clone())
            }
        }
        None
    }
}

impl TryFrom<&str> for Variable {
    type Error = parser::ParseErr;

    fn try_from(vn: &str) -> Result<Self, Self::Error> {
        parser::parse_variable(None, vn)
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

impl TryFrom<&str> for Type {
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

fn is_free(v: &Variable, ctx: &[String]) -> bool {
    let mut layers_left = v.layer;

    for name in ctx {
        if name == &v.name {
            if layers_left == 0 {
                return false;
            } else {
                layers_left -= 1
            }
        }
    }

    true
}

impl Term {
    pub fn free_vars(&self) -> HashSet<Variable> {
        self.free_vars_in_ctx(&[])
    }

    fn free_vars_in_ctx(&self, ctx: &[String]) -> HashSet<Variable> {
        use TermNode::*;

        match self.as_node() {
            Var(v) => {
                let mut free_vars = HashSet::new();
                if is_free(&v, ctx) {
                    free_vars.insert(v.clone());
                }
                free_vars
            },
            Lam(x, t) => {
                let mut new_ctx = ctx.to_owned();
                new_ctx.push(x.clone());
                t.free_vars_in_ctx(&new_ctx)
            },
            App(t, vs) => {
                let mut free_vars = HashSet::new();

                for free_var in t.free_vars_in_ctx(ctx) {
                    free_vars.insert(free_var);
                }

                for v in vs {
                    for free_var in v.free_vars_in_ctx(ctx) {
                        free_vars.insert(free_var);
                    }
                }

                free_vars
            },
            Let(x, v, t) => {
                let mut free_vars: HashSet<Variable> = v.free_vars_in_ctx(ctx).iter().cloned().collect();

                let mut new_ctx = ctx.to_owned();
                new_ctx.push(x.clone());

                free_vars.extend(t.free_vars_in_ctx(&new_ctx).iter().cloned());
                free_vars.iter().cloned().collect()
            },
            Match(t, match_arms) => {
                let mut free_vars = HashSet::new();
                for fv in t.free_vars_in_ctx(ctx) {
                    free_vars.insert(fv);
                }

                for MatchArm(pat, body) in match_arms {
                    let mut new_ctx = ctx.to_owned();
                    new_ctx.extend(pat[1..].iter().cloned());

                    for fv in body.free_vars_in_ctx(&new_ctx) {
                        free_vars.insert(fv);
                    }
                }

                free_vars.iter().cloned().collect()
            },
            Hole(_) => HashSet::new(),
            As(t, _ty) => t.free_vars_in_ctx(ctx),
            StrLit(_s) => HashSet::new(),
        }
    }

    pub fn as_node(&self) -> &TermNode {
        let Term(node) = self;
        node
    }
}
