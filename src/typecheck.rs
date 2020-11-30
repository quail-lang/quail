use std::rc;

use super::ast::Term;
use super::ast::TermNode;

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

pub fn infer_type(t: Term, ctx: TypeContext) -> Result<Type, TypeErr> {
    match t.as_ref() {
        TermNode::Var(x) => {
            match ctx.lookup(x) {
                None => Err(format!("Variable {} not found in context", &x)),
                Some(typ) => Ok(typ),
            }
        },
        TermNode::Lam(_y, _body) => Err("Can't infer type of functions.".to_string()),
        TermNode::App(f, vs) => {
            let mut result = infer_type(f.clone(), ctx.clone())?;

            for v in vs.iter() {
                match result.as_ref() {
                    TypeNode::Atom(_) => return Err("Expected function type.".to_string()),
                    TypeNode::Arrow(dom, cod) => {
                        check_type(v.clone(), ctx.clone(), dom.clone())?;
                        result = cod.clone();
                    },
                }
            }
            Ok(result)
        },
        TermNode::Let(x, v, body) => {
            let x_typ = infer_type(v.clone(), ctx.clone())?;
            infer_type(body.clone(), ctx.extend(x, x_typ))
        },
        TermNode::Match(_t, _match_arms) => {
            Err("Can't infer type of match statements. (Yet?)".to_string())
        },
        TermNode::Hole(_hole_info) => Err("Can't infer type of a hole.".to_string()),
        TermNode::As(term, typ) => {
            check_type(term.clone(), ctx, typ.clone())?;
            Ok(typ.clone())
        },
    }
}

pub fn check_type(t: Term, ctx: TypeContext, typ: Type) -> Result<(), TypeErr> {
    match t.as_ref() {
        TermNode::Var(x) => {
            match ctx.lookup(&x) {
                Some(x_typ) => {
                    if x_typ == typ {
                        Ok(())
                    } else {
                        Err(format!("Type of {} does not match context", x))
                    }
                },
                None => Err(format!("{} does not appear in context.", x)),
            }
        },
        TermNode::Lam(x, body) => {
            match typ.as_ref() {
                TypeNode::Atom(atom) => Err(format!("Functions need function types, but we got {}", atom)),
                TypeNode::Arrow(dom, cod) => check_type(body.clone(), ctx.extend(x, dom.clone()), cod.clone()),
            }
        },
        TermNode::App(_f, _vs) => {
            let inferred_typ = infer_type(t.clone(), ctx)?;
            if &inferred_typ == &typ {
                Ok(())
            } else {
                Err(format!("Type mismatch during application: {:?} vs {:?}", &inferred_typ, &typ))
            }
        },
        TermNode::Let(x, v, body) => {
            let x_typ = infer_type(v.clone(), ctx.clone())?;
            check_type(body.clone(), ctx.extend(x, x_typ), typ)
        },
        TermNode::Match(_t, _match_arms) => {
            unimplemented!()
        },
        TermNode::Hole(_hole_info) => Ok(()),
        TermNode::As(term, as_typ) => {
            if &typ == as_typ {
                check_type(term.clone(), ctx, typ)
            } else {
                Err(format!("Type mismatch during ascription: {:?} vs {:?}", &as_typ, &typ))
            }
        },
    }
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

/*
fn type_of_ctx(t: Term, ctx: HashMap<String, Type>) -> Type {
    match t.as_ref() {
        ast::TermNode::Var(y) => {
            ctx.get(y).unwrap().clone()
        }
        ast::TermNode::Lam(y, body) => {
            // TODO
            let mut new_ctx = ctx.clone();
            new_ctx.insert(y.clone(), ty.clone());
            let cod = type_of_ctx(body.clone(), new_ctx);
            ast::Type::Arrow(Box::new(ty.clone()), Box::new(cod))
        }
        ast::TermNode::App(f, w) => {
            // TODO
            let f_ty = type_of_ctx(f.clone(), ctx.clone());
            let _w_ty = type_of_ctx(w.clone(), ctx.clone());
            if let ast::Type::Arrow(_dom, cod) = f_ty {
                *cod
            } else {
                panic!("Impossible"); // TODO
            }
        },
        ast::TermNode::BoolLit(_b) => Type::Bool,
        ast::TermNode::NatLit(_n) => Type::Nat,
        ast::TermNode::PrimApp(f, vs) => type_of_prim(*f, vs.clone()),
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
*/
