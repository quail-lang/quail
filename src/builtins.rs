use std::collections::HashMap;
use std::rc;

use crate::ast;

use ast::Value;
use ast::Context;
use crate::typecheck::TypeNode;
use crate::typecheck::TypeContext;
use crate::typecheck::Type;
use crate::ast::CtorTag;

#[derive(Debug)]
pub struct InductiveTypeDef {
    name: String,
    ctor_types: HashMap<CtorTag, Type>,
}

impl InductiveTypeDef {
    pub fn new(name: &str, ctor_signatures: &[(CtorTag, &[Type])]) -> Self {
        let mut ctor_types = HashMap::new();
        for (tag, typ) in ctor_signatures.into_iter().map(|(tag, sig)| (tag, ctor_type_from_signature(&name, &sig))) {
            ctor_types.insert(tag.to_string(), typ);
        }

        InductiveTypeDef {
            name: name.to_string(),
            ctor_types,
        }
    }

    pub fn as_type(&self) -> Type {
        TypeNode::Atom(self.name.clone()).into()
    }

    pub fn ctor_context(&self) -> TypeContext {
        let ctor_types: Vec<(&CtorTag, &Type)> = self.ctor_types.iter().collect();
        let mut ctx = TypeContext::empty();
        for (tag, typ) in ctor_types {
            ctx = ctx.extend(tag, typ.clone())
        }
        ctx
    }

}

fn ctor_type_from_signature(name: &str, ctor_signature: &[Type]) -> Type {
    let mut typ: Type = TypeNode::Atom(name.to_string()).into();
    for sig_typ in ctor_signature.iter().rev() {
        typ = TypeNode::Arrow(sig_typ.clone(), typ).into();
    }
    typ
}

pub fn builtin_inductive_typedefs() -> Vec<InductiveTypeDef> {
    let nat_type = InductiveTypeDef::new(
        "Nat",
        &[
            ("zero".to_string(), &[]),
            ("succ".to_string(), &[TypeNode::Atom("Nat".to_string()).into()]),
        ]
    );

    let bool_type = InductiveTypeDef::new(
        "Bool",
        &[
            ("true".to_string(), &[]),
            ("false".to_string(), &[]),
        ],
    );

    let unit_type = InductiveTypeDef::new(
        "Unit",
        &[
            ("unit".to_string(), &[]),
        ],
    );

    vec![nat_type, bool_type, unit_type]
}

pub fn builtins_ctx() -> Context {
    Context::empty()
        .extend("println", Value::Prim(rc::Rc::new(Box::new(println_prim))))
        .extend("show", Value::Prim(rc::Rc::new(Box::new(show_prim))))
        .extend("zero", Value::Ctor("zero".into(), Vec::new()))
        .extend("succ", Value::Ctor("succ".into(), Vec::new()))
        .extend("true", Value::Ctor("true".into(), Vec::new()))
        .extend("false", Value::Ctor("false".into(), Vec::new()))
        .extend("nil", Value::Ctor("nil".into(), Vec::new()))
        .extend("cons", Value::Ctor("cons".into(), Vec::new()))
        .extend("unit", Value::Ctor("unit".into(), Vec::new()))
        .extend("pair", Value::Ctor("pair".into(), Vec::new()))
}

pub fn builtins_type_ctx() -> TypeContext {
    TypeContext::empty()
        .extend("println", TypeNode::Arrow(
                TypeNode::Atom("Str".to_string()).into(),
                TypeNode::Atom("Unit".to_string()).into(),
        ).into())
        .extend("zero", TypeNode::Atom("Nat".to_string()).into())
        .extend("succ", TypeNode::Arrow(TypeNode::Atom("Nat".to_string()).into(), TypeNode::Atom("Nat".to_string()).into()).into())
        .extend("true", TypeNode::Atom("Bool".to_string()).into())
        .extend("false", TypeNode::Atom("Bool".to_string()).into())
        .extend("unit", TypeNode::Atom("Unit".to_string()).into())
        .extend("show", TypeNode::Arrow(TypeNode::Atom("Nat".to_string()).into(), TypeNode::Atom("Str".to_string()).into()).into())
}

fn println_prim(vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "println must have exactly one argument");
    let v = vs[0].clone();
    println!("{:?}", v);
    Value::Ctor("unit".into(), Vec::new())
}

fn show_prim(vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "show must have exactly one argument");
    let v = vs[0].clone();
    match &v {
        Value::Ctor(tag, _) => {
            if tag == "zero" || tag == "succ" {
                Value::Str(format!("{}", nat_to_u64(v)))
            } else if tag == "nil" || tag == "cons" {
                let val_vec = list_to_vec(v.clone());
                let str_value_vec: Vec<Value> = val_vec.into_iter().map(|v| show_prim(vec![v])).collect();
                let s: String = format!("{:?}", str_value_vec);
                Value::Str(s)
            } else {
                Value::Str(format!("{:?}", v))
            }
        }
        _ => panic!("Can't show this {:?}", &v),
    }
}

fn list_to_vec(v: Value) -> Vec<Value> {
    match v {
        Value::Ctor(tag, contents) => {
            if tag == "nil" {
                Vec::new()
            } else if tag == "cons" {
                let head = &contents[0];
                let tail = &contents[1];
                let mut result = list_to_vec(tail.clone());
                result.insert(0, head.clone());
                result
            } else {
                 panic!("This isn't a list.")
            }
        },
        _ => panic!("This isn't a list."),
    }
}

fn nat_to_u64(v: Value) -> u64 {
    match v {
        Value::Ctor(tag, contents) => {
            if tag == "zero" {
                0
            } else if tag == "succ" {
                let inner_value = &contents[0];
                1 + nat_to_u64(inner_value.clone())
            } else {
                 panic!("This isn't a nat.")
            }
        },
        _ => panic!("This isn't a nat."),
    }
}
