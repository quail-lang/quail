use std::collections::HashMap;
use std::rc;

use crate::runtime::Value;
use crate::runtime::Context;
use crate::ast::Tag;
use crate::runtime::Runtime;
use crate::types::Type;
use crate::types::TypeNode;
use crate::types::context::TypeContext;

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

type PrimCode = Box<dyn Fn(&mut Runtime, Vec<Value>) -> Value>;

pub struct PrimDef {
    pub name: String,
    pub typ: Type,
    pub code: PrimCode,
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
    pub fn new(name: &str, flavor: Flavor, ctor_signatures: &[(Tag, Vec<Type>)]) -> Self {
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


impl PrimDef {
    pub fn new(name: String, typ: Type, code: PrimCode) -> Self {
        PrimDef {
            name,
            typ,
            code,
        }
    }
}

fn ctor_type_from_signature(name: &str, ctor_signature: &[Type]) -> Type {
    let mut typ: Type = TypeNode::Atom(name.to_string()).into();
    for sig_typ in ctor_signature.iter().rev() {
        typ = TypeNode::Arrow(sig_typ.clone(), typ).into();
    }
    typ
}

pub fn parse_typedef(types_text_lines: &mut dyn Iterator<Item=&str>) -> Option<TypeDef> {
    let first_line = types_text_lines.next()?;
    let flavor = match first_line.split(" ").next()? {
        "inductive" => Flavor::Inductive,
        "coinductive" => Flavor::Coinductive,
        _ => panic!("Illegal flavor in type declaration: {}", first_line),
    };

    let mut ctors: Vec<(String, Vec<Type>)> = Vec::new();

    while let Some(line) = types_text_lines.next() {
        let line = line.trim();
        if line == "" {
            break;
        }

        let parts: Vec<&str> = line.split(" ").collect();
        let (ctor_name, arg_type_names) = parts.split_first()?;

        let mut arg_types = Vec::new();
        for arg_type_name in arg_type_names {
            arg_types.push(TypeNode::Atom(arg_type_name.to_string()).into());
        }

        let ctor = (ctor_name.to_string(), arg_types);
        ctors.push(ctor);
    }

    let name = first_line.split(" ").last()?.trim();
    let typedef = TypeDef::new(name, flavor, ctors.as_slice());
    Some(typedef)
}

///
/// Returns a list of inductive typedefs which are considered "built-in" in Quail.
///
pub fn builtin_inductive_typedefs() -> Vec<TypeDef> {
    let mut typedefs = vec![];
    let mut types_text_lines = include_str!("../../assets/types.txt").lines();
    while let Some(typedef) = parse_typedef(&mut types_text_lines) {
        typedefs.push(typedef);
    }

    typedefs
}

pub fn builtin_primdefs() -> Vec<PrimDef> {
    let println_primdef = PrimDef::new(
        "println".to_string(),
          TypeNode::Arrow(
            TypeNode::Atom("Str".to_string()).into(),
            TypeNode::Atom("Top".to_string()).into(),
        ).into(),
        Box::new(println_prim),
    );

    let show_primdef = PrimDef::new(
        "show".to_string(),
        TypeNode::Arrow(
            TypeNode::Atom("Nat".to_string()).into(),
            TypeNode::Atom("Str".to_string()).into(),
        ).into(),
        Box::new(show_prim),
    );

    let show_list_primdef = PrimDef::new(
        "show_list".to_string(),
        TypeNode::Arrow(
            TypeNode::Atom("List".to_string()).into(),
            TypeNode::Atom("Str".to_string()).into(),
        ).into(),
        Box::new(show_list_prim),
    );

    let cat_primdef = PrimDef::new(
        "cat".to_string(),
        TypeNode::Arrow(
            TypeNode::Atom("Str".to_string()).into(),
            TypeNode::Arrow(
                TypeNode::Atom("Str".to_string()).into(),
                TypeNode::Atom("Str".to_string()).into(),
            ).into(),
        ).into(),
        Box::new(cat_prim),
    );

    vec![println_primdef, show_primdef, show_list_primdef, cat_primdef]
}

pub fn builtins_ctx() -> Context {
    let mut ctx = Context::empty();

    for primdef in builtin_primdefs() {
        ctx = ctx.extend(
            &primdef.name.to_string(),
            Value::Prim(rc::Rc::new(primdef.code)),
        );
    }
    ctx
}

pub fn builtins_type_ctx() -> TypeContext {
    let mut ctx = TypeContext::empty();

    for primdef in builtin_primdefs() {
        ctx = ctx.extend(
            &primdef.name.to_string(),
            primdef.typ,
        );
    }
    ctx
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

fn show_list_prim(runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "show_list must have exactly one argument");
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
    let mut val = v;
    let mut result = 0;

    loop {
        match val {
            Value::Ctor(tag, contents) => {
                if tag == "zero" {
                    break
                } else if tag == "succ" {
                    val = contents[0].clone();
                    result += 1;
                } else {
                     panic!("This isn't a nat.")
                }
            },
            _ => panic!("This isn't a nat."),
        }
    }

    result
}

fn cat_prim(_runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 2, "show_list must have exactly two arguments");
    let v1 = vs[0].clone();
    let v2 = vs[1].clone();
    match (&v1, &v2) {
        (Value::Str(s1), Value::Str(s2)) => Value::Str(format!("{}{}", s1, s2)),
        _ => panic!("Arguments to cat must both be Str: {:?} {:?}", &v1, &v2),
    }
}