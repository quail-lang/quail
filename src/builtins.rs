use std::rc;

use crate::ast;

use ast::Value;
use ast::Context;
use crate::typecheck::TypeNode;
use crate::typecheck::TypeContext;

pub fn builtins_ctx() -> Context {
    Context::empty()
        .extend("println", Value::Prim(rc::Rc::new(Box::new(println_prim))))
        .extend("zero", Value::Ctor("zero".into(), Vec::new()))
        .extend("succ", Value::Prim(rc::Rc::new(Box::new(succ_prim))))
        .extend("true", Value::Ctor("true".into(), Vec::new()))
        .extend("false", Value::Ctor("false".into(), Vec::new()))
        .extend("nil", Value::Ctor("nil".into(), Vec::new()))
        .extend("cons", Value::Prim(rc::Rc::new(Box::new(cons_prim))))
        .extend("unit", Value::Ctor("unit".into(), Vec::new()))
        .extend("pair", Value::Prim(rc::Rc::new(Box::new(pair_prim))))
        .extend("show", Value::Prim(rc::Rc::new(Box::new(show_prim))))
}

pub fn builtins_type_ctx() -> TypeContext {
    TypeContext::empty()
        .extend("println", TypeNode::Atom("Unit".to_string()).into())
        .extend("zero", TypeNode::Atom("Nat".to_string()).into())
        .extend("succ", TypeNode::Arrow(TypeNode::Atom("Nat".to_string()).into(), TypeNode::Atom("Nat".to_string()).into()).into())
        .extend("true", TypeNode::Atom("Bool".to_string()).into())
        .extend("false", TypeNode::Atom("Bool".to_string()).into())
        .extend("unit", TypeNode::Atom("Unit".to_string()).into())
}

fn succ_prim(vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "succ must have exactly one argument");
    let v = vs[0].clone();
    match &v {
        Value::Ctor(tag, _) => {
            if tag == "zero" {
                Value::Ctor("succ".into(), vec![Value::Ctor("zero".into(), vec![])])
            } else if tag == "succ" {
                Value::Ctor("succ".into(), vec![v.clone()])
            } else {
                panic!("Invalid thing to succ: {:?}", &v)
            }
        },
        other => panic!(format!("Couldn't succ {:?}", other)),
    }
}

fn cons_prim(vs: Vec<Value>) -> Value {
    let head = vs[0].clone();
    let tail = vs[1].clone();
    Value::Ctor("cons".into(), vec![head, tail])
}

fn pair_prim(vs: Vec<Value>) -> Value {
    let fst = vs[0].clone();
    let snd = vs[1].clone();
    Value::Ctor("pair".into(), vec![fst, snd])
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
