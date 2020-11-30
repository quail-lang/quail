use std::fmt;
use std::rc;
use std::collections::HashSet;
use std::collections::HashMap;

use super::ast::Term;
use super::ast::TermNode;
use crate::ast::CtorTag;
use crate::ast::MatchArm;
use crate::builtins::InductiveTypeDef;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type(pub rc::Rc<TypeNode>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeNode {
    Atom(String),
    Arrow(Type, Type),
}

#[derive(Debug)]
struct TypeContextNode(Vec<(String, Type)>);

#[derive(Debug, Clone)]
pub struct TypeContext(rc::Rc<TypeContextNode>);

pub type TypeErr = String;

pub fn infer_type(t: Term, ctx: TypeContext, inductive_typedefs: &HashMap<String, InductiveTypeDef>) -> Result<Type, TypeErr> {
    match t.as_ref() {
        TermNode::Var(x) => {
            match ctx.lookup(x) {
                None => Err(format!("Variable {} not found in context", &x)),
                Some(typ) => Ok(typ),
            }
        },
        TermNode::Lam(_y, _body) => Err("Can't infer type of functions.".to_string()),
        TermNode::App(f, vs) => {
            let mut result = infer_type(f.clone(), ctx.clone(), inductive_typedefs)?;

            for v in vs.iter() {
                match result.as_ref() {
                    TypeNode::Atom(_) => return Err("Expected function type.".to_string()),
                    TypeNode::Arrow(dom, cod) => {
                        check_type(v.clone(), ctx.clone(), inductive_typedefs, dom.clone())?;
                        result = cod.clone();
                    },
                }
            }
            Ok(result)
        },
        TermNode::Let(x, v, body) => {
            let x_typ = infer_type(v.clone(), ctx.clone(), inductive_typedefs)?;
            infer_type(body.clone(), ctx.extend(x, x_typ), inductive_typedefs)
        },
        TermNode::Match(_t, _match_arms) => {
            Err("Can't infer type of match statements. (Yet?)".to_string())
        },
        TermNode::Hole(_hole_info) => Err("Can't infer type of a hole.".to_string()),
        TermNode::As(term, typ) => {
            check_type(term.clone(), ctx, inductive_typedefs, typ.clone())?;
            Ok(typ.clone())
        },
    }
}

pub fn check_type(t: Term, ctx: TypeContext, inductive_typedefs: &HashMap<String, InductiveTypeDef>, typ: Type) -> Result<(), TypeErr> {
    match t.as_ref() {
        TermNode::Var(x) => {
            match ctx.lookup(&x) {
                Some(x_typ) => {
                    if x_typ == typ {
                        Ok(())
                    } else {
                        Err(format!("Term {} does not have type {:?} in context: {:?}", x, &typ, &ctx))
                    }
                },
                None => Err(format!("{} does not appear in context.", x)),
            }
        },
        TermNode::Lam(x, body) => {
            match typ.as_ref() {
                TypeNode::Atom(atom) => Err(format!("Functions need function types, but we got {}", atom)),
                TypeNode::Arrow(dom, cod) => check_type(body.clone(), ctx.extend(x, dom.clone()), inductive_typedefs, cod.clone()),
            }
        },
        TermNode::App(_f, _vs) => {
            let inferred_typ = infer_type(t.clone(), ctx, inductive_typedefs)?;
            if &inferred_typ == &typ {
                Ok(())
            } else {
                Err(format!("Type mismatch during application: {:?} vs {:?}", &inferred_typ, &typ))
            }
        },
        TermNode::Let(x, v, body) => {
            let x_typ = infer_type(v.clone(), ctx.clone(), inductive_typedefs)?;
            check_type(body.clone(), ctx.extend(x, x_typ), inductive_typedefs, typ)
        },
        TermNode::Match(t, match_arms) => check_type_match(t, match_arms, ctx, inductive_typedefs, typ),
        TermNode::Hole(_hole_info) => Ok(()),
        TermNode::As(term, as_typ) => {
            if &typ == as_typ {
                check_type(term.clone(), ctx, inductive_typedefs, typ)
            } else {
                Err(format!("Type mismatch during ascription: {:?} vs {:?}", &as_typ, &typ))
            }
        },
    }
}

pub fn check_type_match(
    discriminee: &Term,
    match_arms: &[MatchArm],
    ctx: TypeContext,
    inductive_typedefs: &HashMap<String, InductiveTypeDef>,
    typ: Type,
) -> Result<(), TypeErr> {
    let match_tags: Vec<CtorTag> = match_arms.iter().map(|MatchArm(pat, _arm_term)| pat[0].to_string()).collect();
    // TODO: handle bottom type
    let first_ctor_tag = &match_tags.iter().cloned().collect::<Vec<CtorTag>>()[0];
    match lookup_typedef_by_ctor_tag(first_ctor_tag, inductive_typedefs) {
        None => Err(format!("Unknown ctor {:?}", first_ctor_tag)),
        Some(inductive_typedef) => {
            let typedef_tags = inductive_typedef.ctor_tags();
            analyze_coverage(&typedef_tags, &match_tags)?;
            check_type(discriminee.clone(), ctx.clone(), inductive_typedefs, TypeNode::Atom(inductive_typedef.name.to_string()).into())?;
            for match_arm in match_arms {
                check_type_match_arm(match_arm, inductive_typedef, &ctx, inductive_typedefs, &typ)?;
            }
            Ok(())
        },
    }
}

fn analyze_coverage(typedef_tags: &Vec<CtorTag>, match_tags: &Vec<CtorTag>) -> Result<(), TypeErr> {
    let match_tags_set: HashSet<_> = match_tags.iter().cloned().collect();
    let typedef_tags_set: HashSet<_> = typedef_tags.iter().cloned().collect();

    let unexpected_tags: HashSet<_> = match_tags_set.difference(&typedef_tags_set).collect();
    let missing_tags: HashSet<_> = typedef_tags_set.difference(&match_tags_set).collect();

    let mut sorted_match_tags = match_tags.clone();
    sorted_match_tags.sort();
    let mut duplicate_tags: HashSet<CtorTag> = HashSet::new();
    let match_tag_with_next: Vec<_> = sorted_match_tags
        .iter()
        .zip(sorted_match_tags[1..].iter())
        .collect();

    for (cur, next) in match_tag_with_next.into_iter() {
        if cur == next {
            duplicate_tags.insert(cur.to_string());
        }
    }

    if !unexpected_tags.is_empty() {
        Err(format!("Found unexpected tags: {:?}", unexpected_tags))
    } else if !missing_tags.is_empty() {
        Err(format!("Expected missing tags: {:?}", missing_tags))
    } else if !duplicate_tags.is_empty() {
        Err(format!("Duplicate tags: {:?}", duplicate_tags))
    } else {
        Ok(())
    }
}

fn check_type_match_arm(
    match_arm: &MatchArm,
    inductive_typedef: &InductiveTypeDef,
    ctx: &TypeContext,
    inductive_typedefs: &HashMap<String, InductiveTypeDef>,
    typ: &Type,
) -> Result<(), TypeErr> {
    let MatchArm(pat, body) = match_arm;
    let ctor_tag = pat[0].to_string();
    let mut ctor_typ = inductive_typedef.ctor_types.get(&ctor_tag).unwrap();

    let pattern_names: Vec<String> = (&pat[1..]).iter().cloned().collect();
    let mut pattern_types: Vec<Type> = Vec::new();

    while let TypeNode::Arrow(dom, cod) = ctor_typ.as_ref() {
        pattern_types.push(dom.clone());
        ctor_typ = cod;
    }

    if pattern_names.len() != pattern_types.len() {
        Err(format!("Pattern has the wrong number of variables: {:?} is more than {}", pattern_names, pattern_types.len()))
    } else {
        let zipped: Vec<(String, Type)> = pattern_names.into_iter().zip(pattern_types).collect();
        let extended_ctx = ctx.extend_many(&zipped);
        check_type(body.clone(), extended_ctx.clone(), inductive_typedefs, typ.clone())
    }
}

fn lookup_typedef_by_ctor_tag<'a>(ctor_tag: &CtorTag, inductive_typedefs: &'a HashMap<String, InductiveTypeDef>) -> Option<&'a InductiveTypeDef> {
    for (_typename, inductive_typedef) in inductive_typedefs.iter() {
        let ctor_tags: Vec<CtorTag> = inductive_typedef.ctor_types.keys().cloned().collect();
        if ctor_tags.contains(&ctor_tag) {
            return Some(inductive_typedef);
        }
    }
    None
}

impl From<TypeNode> for Type {
    fn from(tn: TypeNode) -> Self {
        Type(rc::Rc::new(tn))
    }
}

impl TypeContext {
    pub fn empty() -> Self {
        TypeContext(rc::Rc::new(TypeContextNode(Vec::new())))
    }

    pub fn lookup(&self, x: &str) -> Option<Type> {
        let TypeContext(rc_ctx_node) = self;
        let TypeContextNode(var_typ_list) = rc_ctx_node.as_ref();
        for (y, typ) in var_typ_list.iter().rev() {
            if x == y {
                return Some(typ.clone());
            }
        }
        None
    }

    pub fn extend(&self, x: &str, t: Type) -> TypeContext {
        let TypeContext(rc_ctx_node) = self;
        let TypeContextNode(var_val_list) = rc_ctx_node.as_ref();
        let mut extended_var_val_list = var_val_list.clone();
        extended_var_val_list.push((x.to_string(), t.clone()));
        TypeContext(rc::Rc::new(TypeContextNode(extended_var_val_list)))
    }

    pub fn extend_many(&self, bindings: &[(String, Type)]) -> TypeContext {
        let mut ctx = self.clone();
        for (name, value) in bindings.iter() {
            ctx = ctx.extend(name, value.clone());
        }
        ctx
    }

    pub fn append(&self, ctx: TypeContext) -> TypeContext {
        let mut result_ctx = self.clone();
        for (name, value) in ctx.bindings().iter() {
            result_ctx = result_ctx.extend(name, value.clone());
        }
        result_ctx
    }

    pub fn bindings(&self) -> Vec<(String, Type)> {
        let TypeContext(context_node_rc) = self;
        let TypeContextNode(bindings) = context_node_rc.as_ref();
        bindings.clone()
    }
}

impl AsRef<TypeNode> for Type {
    fn as_ref(&self) -> &TypeNode {
        use std::borrow::Borrow;
        let Type(rc_tn) = self;
        rc_tn.borrow()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            TypeNode::Atom(atom) => write!(f, "{}", atom),
            TypeNode::Arrow(dom, cod) => {
                if let TypeNode::Atom(_) = dom.as_ref() {
                    write!(f, "{}", dom)?;
                } else {
                    write!(f, "({})", dom)?;
                }
                write!(f, " -> ")?;
                write!(f, "{}", cod)
            }
        }
    }
}
