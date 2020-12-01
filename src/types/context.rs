use std::rc;
use crate::ast::Type;

#[derive(Debug)]
struct TypeContextNode(Vec<(String, Type)>);

#[derive(Debug, Clone)]
pub struct TypeContext(rc::Rc<TypeContextNode>);

impl TypeContext {
    pub fn empty() -> Self {
        TypeContext(rc::Rc::new(TypeContextNode(Vec::new())))
    }

    pub fn lookup(&self, x: &str, k: usize) -> Option<Type> {
        let TypeContext(rc_ctx_node) = self;
        let TypeContextNode(var_typ_list) = rc_ctx_node.as_ref();
        for (y, typ) in var_typ_list.iter().rev() {
            if x == y {
                if k == 0 {
                    return Some(typ.clone());
                } else {
                    return self.lookup(x, k - 1);
                }
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
