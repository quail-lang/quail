use std::rc;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct Context(rc::Rc<ContextNode>);

impl Context {
    pub fn empty() -> Self {
        Context(rc::Rc::new(ContextNode(Vec::new())))
    }

    pub fn lookup(&self, x: &str, k: usize) -> Option<Value> {
        let Context(rc_ctx_node) = self;
        let ContextNode(var_val_list) = rc_ctx_node.as_ref();
        for (y, value) in var_val_list.iter().rev() {
            if x == y {
                if k == 0 {
                    return Some(value.clone());
                } else {
                    return self.lookup(x, k - 1);
                }
            }
        }
        None
    }

    pub fn extend(&self, x: &str, v: Value) -> Context {
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

    pub fn append(&self, ctx: Context) -> Context {
        let mut result_ctx = self.clone();
        for (name, value) in ctx.bindings().iter() {
            result_ctx = result_ctx.extend(name, value.clone());
        }
        result_ctx
    }

    pub fn bindings(&self) -> Vec<(String, Value)> {
        let Context(context_node_rc) = self;
        let ContextNode(bindings) = context_node_rc.as_ref();
        bindings.clone()
    }
}

#[derive(Debug)]
struct ContextNode(Vec<(String, Value)>);
