use std::collections::HashMap;
use std::rc;

use crate::ast;

use ast::Value;
use ast::Context;
use crate::ast::Tag;
use crate::runtime::Runtime;
use crate::types::Type;
use crate::types::TypeNode;
use crate::typecontext::TypeContext;

#[derive(Debug, Clone)]
pub enum Flavor {
    Inductive,
    Coinductive,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub flavor: Flavor,
    pub ctor_types: HashMap<Tag, Type>,
}

///
/// Inductive Types consist of a name (such as `Nat`) and a list of constructor tags
/// (`zero` and `succ`) together with their types (`Nat` and `Nat -> Nat`).
///
impl TypeDef {
    ///
    /// Creates a new InductiveTypeDef from a name and a list of pairs (tagname, signature).
    /// A signature means the types arguments to the constructor strung together (omitting
    /// the return value, which is inferred to be the inductive type itself).. For example,
    /// if our constructor is `cons`, we would include ("cons", ["Nat", "List"]).
    ///
    pub fn new(name: &str, flavor: Flavor, ctor_signatures: &[(Tag, &[Type])]) -> Self {
        let mut ctor_types = HashMap::new();
        for (tag, typ) in ctor_signatures.into_iter().map(|(tag, sig)| (tag, ctor_type_from_signature(&name, &sig))) {
            ctor_types.insert(tag.to_string(), typ);
        }

        TypeDef {
            name: name.to_string(),
            flavor,
            ctor_types,
        }
    }

    ///
    /// Create a value-level context containing the constructors for this inductive type.
    ///
    pub fn ctor_context(&self) -> Context {
        let ctors: Vec<&Tag> = self.ctor_types.keys().collect();
        let mut ctx = Context::empty();
        for tag in ctors {
            match self.flavor {
                Flavor::Inductive => ctx = ctx.extend(tag, Value::Ctor(tag.to_string(), vec![])),
                Flavor::Coinductive => ctx = ctx.extend(tag, Value::CoCtor(tag.to_string(), vec![])),
            }
        }
        ctx
    }

    ///
    /// Create a type-level context containing the constructors for this inductive type.
    ///
    pub fn ctor_type_context(&self) -> TypeContext {
        let ctor_types: Vec<(&Tag, &Type)> = self.ctor_types.iter().collect();
        let mut ctx = TypeContext::empty();
        for (tag, typ) in ctor_types {
            ctx = ctx.extend(tag, typ.clone())
        }
        ctx
    }

    ///
    /// Return the list of tagnames for the inductive type.
    ///
    pub fn ctor_tags(&self) -> Vec<Tag> {
        self.ctor_types.keys().cloned().collect()
    }
}

fn ctor_type_from_signature(name: &str, ctor_signature: &[Type]) -> Type {
    let mut typ: Type = TypeNode::Atom(name.to_string()).into();
    for sig_typ in ctor_signature.iter().rev() {
        typ = TypeNode::Arrow(sig_typ.clone(), typ).into();
    }
    typ
}

///
/// Returns a list of inductive typedefs which are considered "built-in" in Quail.
///
pub fn builtin_inductive_typedefs() -> Vec<TypeDef> {
    let nat_type = TypeDef::new(
        "Nat",
        Flavor::Inductive,
        &[
            ("zero".to_string(), &[]),
            ("succ".to_string(), &[TypeNode::Atom("Nat".to_string()).into()]),
        ]
    );

    let bool_type = TypeDef::new(
        "Bool",
        Flavor::Inductive,
        &[
            ("true".to_string(), &[]),
            ("false".to_string(), &[]),
        ],
    );

    let top_type = TypeDef::new(
        "Top",
        Flavor::Inductive,
        &[
            ("top".to_string(), &[]),
        ],
    );

    let bot_type = TypeDef::new(
        "Bot",
        Flavor::Inductive,
        &[],
    );

    let list_type = TypeDef::new(
        "List",
        Flavor::Inductive,
        &[
            ("nil".to_string(), &[]),
            ("cons".to_string(), &[
                TypeNode::Atom("Nat".to_string()).into(),
                TypeNode::Atom("List".to_string()).into(),
            ]),
        ],
    );

    let conat_type = TypeDef::new(
        "CoNat",
        Flavor::Coinductive,
        &[
            ("cozero".to_string(), &[]),
            ("cosucc".to_string(), &[
                TypeNode::Atom("CoNat".to_string()).into(),
            ]),
        ],
    );

    vec![nat_type, bool_type, top_type, bot_type, list_type, conat_type]
}

pub fn builtins_ctx() -> Context {
    Context::empty()
        .extend("println", Value::Prim(rc::Rc::new(Box::new(println_prim))))
        .extend("show", Value::Prim(rc::Rc::new(Box::new(show_prim))))
}

pub fn builtins_type_ctx() -> TypeContext {
    TypeContext::empty()
        .extend("println", TypeNode::Arrow(
                TypeNode::Atom("Str".to_string()).into(),
                TypeNode::Atom("Top".to_string()).into(),
        ).into())
        .extend("show", TypeNode::Arrow(TypeNode::Atom("Nat".to_string()).into(), TypeNode::Atom("Str".to_string()).into()).into())
}

fn println_prim(_runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "println must have exactly one argument");
    let v = vs[0].clone();
    println!("{:?}", v);
    Value::Ctor("unit".into(), Vec::new())
}

fn show_prim(runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "show must have exactly one argument");
    let v = vs[0].clone();
    match &v {
        Value::Ctor(tag, _) => {
            if tag == "zero" || tag == "succ" {
                Value::Str(format!("{}", nat_to_u64(v)))
            } else if tag == "nil" || tag == "cons" {
                let val_vec = list_to_vec(v.clone());
                let str_value_vec: Vec<Value> = val_vec.into_iter()
                    .map(|v| show_prim(runtime, vec![v]))
                    .collect();
                let s: String = format!("{:?}", str_value_vec);
                Value::Str(s)
            } else {
                Value::Str(format!("{:?}", v))
            }
        },
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
