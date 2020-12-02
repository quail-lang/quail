use std::rc::Rc;

pub type Var = String;
pub type Ctor = String;
pub type UpdateFlag = bool;

#[derive(Debug, Clone)]
pub struct Program(pub Vec<Binding>);

#[derive(Debug, Clone)]
pub struct Binding(pub Var, pub LambdaForm);

#[derive(Debug, Clone)]
pub struct LambdaForm(pub Vec<Var>, pub bool, pub Vec<Var>, pub Expr);

#[derive(Debug, Clone)]
pub struct Expr(Rc<ExprNode>);

#[derive(Debug, Clone)]
pub enum ExprNode {
    Let(LetType, Vec<Binding>, Expr),
    Case(Expr, Alts),
    App(AppType, Var, Vec<Atom>),
    Lit(usize),
}

#[derive(Debug, Clone)]
pub enum Atom {
    Var(Var),
    Lit(usize),
}

#[derive(Debug, Clone)]
pub enum AppType {
    Fun,
    Ctor,
    Prim,
}

#[derive(Debug, Clone)]
pub enum LetType {
    Recursive,
    NonRecursive,
}

#[derive(Debug, Clone)]
pub enum Alt {
    Ctor(Ctor, Vec<Var>, Expr),
    Lit(usize, Expr),
    Default(Var, Expr),
}

#[derive(Debug, Clone)]
pub struct Alts(pub Vec<Alt>);

impl From<&str> for Atom {
    fn from(name: &str) -> Self {
        Atom::Var(name.to_owned())
    }
}

impl std::ops::Deref for Expr {
    type Target = ExprNode;

    fn deref(&self) -> &Self::Target {
        use std::borrow::Borrow;
        let Expr(r) = self;
        r.borrow()
    }
}

impl AsRef<ExprNode> for Expr {
    fn as_ref(&self) -> &ExprNode {
        use std::borrow::Borrow;
        let Expr(rc_tn) = self;
        rc_tn.borrow()
    }
}

impl From<ExprNode> for Expr {
    fn from(node: ExprNode) -> Self {
        Expr(Rc::new(node))
    }
}

impl std::fmt::Display for ExprNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        pprint_exprnode(f, &self, 0)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_ref())
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for Binding(var, lf) in self.0.iter() {
            write!(f, "{} = ", var)?;
            pprint_lf_header(f, lf)?;
            writeln!(f, " -> ")?;
            write!(f, "    ")?;
            pprint_exprnode(f, &lf.3, 0)?;
            writeln!(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

fn pprint_lf_header(f: &mut std::fmt::Formatter, lf: &LambdaForm) -> Result<(), std::fmt::Error> {
    let LambdaForm(vs, pi, xs, _e) = lf;
    pprint_list(f, vs)?;
    if *pi {
        write!(f, " \\u ")?;
    } else {
        write!(f, " \\n ")?;
    }
    pprint_list(f, xs)
}

fn pprint_indent_prefix(f: &mut std::fmt::Formatter, indent: usize) -> Result<(), std::fmt::Error> {
    for _ in 0..indent {
        write!(f, "    ")?;
    }
    Ok(())
}

fn pprint_exprnode(f: &mut std::fmt::Formatter, e: &ExprNode, indent: usize) -> Result<(), std::fmt::Error> {
    match e {
        ExprNode::App(_app_type, fun, vs) => {
            write!(f, "{} ", fun)?;
            let args = vs.iter().map(|v| {
                match v {
                    Atom::Var(var) => var.to_string(),
                    Atom::Lit(k) => k.to_string(),
                }
            }).collect::<Vec<String>>();
            pprint_list(f, &args)?;
        },
        ExprNode::Let(let_type, bindings, e) => {
            match let_type {
                LetType::NonRecursive => write!(f, "let ")?,
                LetType::Recursive => write!(f, "letrc ")?,
            }

            for (i, Binding(name, lf)) in bindings.iter().enumerate() {
                if i > 0 {
                    pprint_indent_prefix(f, indent + 1)?;
                }
                write!(f, "{} = ", name)?;
                pprint_lf_header(f, lf)?;
                write!(f, " -> ")?;

                let LambdaForm(_vs, _pi, _xs, e) = lf;
                pprint_exprnode(f, e, indent + 1)?;
                writeln!(f)?;
            }
            pprint_indent_prefix(f, indent + 1)?;
            write!(f, "in {}", e.as_ref())?;
        },
        ExprNode::Case(e, alts) => {
            writeln!(f, "case {} of", e.as_ref())?;
            for (i, alt) in alts.0.iter().enumerate() {
                pprint_indent_prefix(f, indent + 1)?;
                match alt {
                    Alt::Ctor(c, xs, e) => {
                        write!(f, "{} ", c)?;
                        pprint_list(f, xs)?;
                        write!(f, " -> ")?;
                        pprint_exprnode(f, e.as_ref(), indent + 1)?;
                    },
                    Alt::Lit(k, e) => {
                        write!(f, "{} -> ", k)?;
                        pprint_exprnode(f, e.as_ref(), indent + 1)?;
                    },
                    Alt::Default(x, e) => {
                        write!(f, "{} -> ", x)?;
                        pprint_exprnode(f, e.as_ref(), indent + 1)?;
                    },
                }

                if i < alts.0.len() {
                    writeln!(f)?;
                }
            }
        },
        ExprNode::Lit(k) => write!(f, "{}", k)?,
    }
    Ok(())
}

fn pprint_list<V: std::fmt::Display>(f: &mut std::fmt::Formatter, vs: &[V]) -> Result<(), std::fmt::Error> {
    write!(f, "{{")?;
    for (i, v) in vs.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }

        write!(f, " {}", v)?
    }
    if vs.len() > 0 {

        write!(f, " ")?
    }
    write!(f, "}}")
}


impl Alts {
    pub fn find_alt_for_ctor(&self, ctor_tag: &Ctor) -> Option<&Alt> {
        for alt in self.0.iter() {
            if let Alt::Ctor(alt_ctor, _vars, _e) = alt {
                if ctor_tag == alt_ctor {
                    return Some(&alt);
                }
            }
        }

        self.find_alt_default()
    }

    pub fn find_alt_for_int(&self,  k: usize) -> Option<&Alt> {
        for alt in self.0.iter() {
            if let Alt::Lit(j, _e) = alt {
                if *j == k {
                    return Some(&alt);
                }
            }
        }

        self.find_alt_default()
    }

    pub fn find_alt_default(&self) -> Option<&Alt> {
        for alt in self.0.iter() {
            if let Alt::Default(_vars, _e) = alt {
                return Some(&alt);
            }
        }
        None
    }

}
