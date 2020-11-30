use std::rc;
use std::fmt;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type(rc::Rc<TypeNode>);


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeNode {
    Atom(String),
    Arrow(Type, Type),
    Forall(String, Type),
}


impl From<TypeNode> for Type {
    fn from(tn: TypeNode) -> Self {
        Type(rc::Rc::new(tn))
    }
}


impl std::ops::Deref for Type {
    type Target = TypeNode;

    fn deref(&self) -> &TypeNode {
        use std::borrow::Borrow;
        let Type(rc_tn) = self;
        rc_tn.borrow()
    }
}

impl AsRef<TypeNode> for Type {
    fn as_ref(&self) -> &TypeNode {
         use std::borrow::Borrow;
         let Type(rc_tn) = self;
         rc_tn.borrow()
    }
}

impl fmt::Display for TypeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeNode::Atom(atom) => write!(f, "{}", atom),
            TypeNode::Arrow(dom, cod) => {
                if let TypeNode::Atom(_) = **dom {
                    write!(f, "{}", **dom)?;
                } else {
                    write!(f, "({})",**dom)?;
                }
                write!(f, " -> ")?;
                write!(f, "{}", **cod)
            }
            TypeNode::Forall(atom, typ) => write!(f, "[{}] -> {}", atom, **typ),
        }
    }
}
