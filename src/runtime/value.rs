use std::fmt;
use std::rc;

use super::Runtime;

use crate::context::Context;
use crate::ast::Tag;
use crate::ast::Term;

#[derive(Clone)]
pub enum Value {
    Ctor(Tag, Vec<Value>),
    CoCtor(Tag, Vec<Value>),
    Fun(String, Term, Context<Value>),
    Prim(rc::Rc<PrimFn>),
    Str(String),
    Thunk(Term, Context<Value>),
}

type PrimFn = dyn Fn(&mut Runtime, Vec<Value>) -> Value;

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Ctor(tag, contents) => {
                write!(f, "{}", &tag)?;
                for value in contents {
                    write!(f, " ({:?})", value)?;
                }
                Ok(())
            },
            Value::CoCtor(tag, contents) => {
                write!(f, "{}", &tag)?;
                for value in contents {
                    write!(f, " ({:?})", value)?;
                }
                Ok(())
            },
            Value::Str(s) => write!(f, "{:?}", s),
            Value::Fun(_, _, _) => write!(f, "<fun>"),
            Value::Prim(_) => write!(f, "<prim>"),
            Value::Thunk(_, _) => write!(f, "<thunk>"),
        }
    }
}
