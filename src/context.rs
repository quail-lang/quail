use std::rc;

#[derive(Debug)]
struct ContextNode<T>(Vec<(String, T)>);

#[derive(Debug, Clone)]
pub struct Context<T>(rc::Rc<ContextNode<T>>);

impl<T: Clone> Context<T> {
    pub fn empty() -> Self {
        Context(rc::Rc::new(ContextNode(Vec::new())))
    }

    pub fn lookup(&self, x: &str, k: usize) -> Option<T> {
        let Context(rc_ctx_node) = self;
        let ContextNode(var_typ_list) = rc_ctx_node.as_ref();
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

    pub fn extend(&self, x: &str, t: T) -> Context<T> {
        let Context(rc_ctx_node) = self;
        let ContextNode(var_val_list) = rc_ctx_node.as_ref();
        let mut extended_var_val_list = var_val_list.clone();
        extended_var_val_list.push((x.to_string(), t.clone()));
        Context(rc::Rc::new(ContextNode(extended_var_val_list)))
    }

    pub fn extend_many(&self, bindings: &[(String, T)]) -> Context<T> {
        let mut ctx = self.clone();
        for (name, value) in bindings.iter() {
            ctx = ctx.extend(name, value.clone());
        }
        ctx
    }

    pub fn append(&self, ctx: Context<T>) -> Context<T> {
        let mut result_ctx = self.clone();
        for (name, value) in ctx.bindings().iter() {
            result_ctx = result_ctx.extend(name, value.clone());
        }
        result_ctx
    }

    pub fn bindings(&self) -> Vec<(String, T)> {
        let Context(context_node_rc) = self;
        let ContextNode(bindings) = context_node_rc.as_ref();
        bindings.clone()
    }
}
