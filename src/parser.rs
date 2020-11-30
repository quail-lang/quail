use crate::ast;
use std::fmt;

pub fn parse(input: impl Into<String>) -> ast::Term {
    let expr = parse_sexpr(input).expect("Could not parse.");
    expr_to_term(expr)
}

fn expr_to_term(expr: Expr) -> ast::Term {
    let Expr(expr_node) = expr;
    match *expr_node {
        ExprNode::List(exprs) => {
            let Expr(head_expr_node) = exprs[0].clone();
            match *head_expr_node {
                ExprNode::Atom(v) => {
                    assert!(exprs.len() == 2);
                    let x = expr_to_term(exprs[1].clone());
                    match v.as_ref() {
                        "fn" => unimplemented!(),
                        vs => ast::TermNode::App(
                            ast::TermNode::Var(vs.to_string()).into(),
                            x,
                        ).into(),
                    }
                },
                f => {
                    assert!(exprs.len() == 2);
                    let x = exprs[1].clone();
                    ast::TermNode::App(
                        expr_to_term(Expr(Box::new(f))),
                        expr_to_term(x),
                    ).into()
                },
            }
        },
        ExprNode::Atom(atom) => match atom.as_ref() {
            "True" => ast::TermNode::BoolLit(true).into(),
            "False" => ast::TermNode::BoolLit(false).into(),
            var_name => ast::TermNode::Var(var_name.to_string()).into(),
        },
        ExprNode::Literal(val) => ast::TermNode::NatLit(val as u64).into(),
    }
}

pub fn parse_sexpr(input: impl Into<String>) -> Option<Expr> {
    Parser::new(input.into()).parse()
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Expr(Box<ExprNode>);

#[derive(PartialEq, Eq, Debug, Clone)]
enum ExprNode {
    List(Vec<Expr>),
    Atom(String),
    Literal(usize),
}

#[derive(PartialEq, Eq, Debug)]
enum ExprType {
    List,
    Atom,
    Literal,
}

struct Parser {
    input: String,
    cursor: usize,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Expr(expr_node) = self;
        match expr_node.as_ref() {
            ExprNode::List(xs) => {
                write!(f, "(")?;
                if xs.len() > 0 {
                    write!(f, "{}", xs[0])?;

                    for x in &xs[1..] {
                        write!(f, " {}", x)?;
                    }
                }
                write!(f, ")")
            },
            ExprNode::Atom(s) => write!(f, "{}", s),
            ExprNode::Literal(l) => write!(f, "{}", l),
        }
    }
}

impl Parser {
    fn new(input: String) -> Parser {
        Parser { input, cursor: 0 }
    }

    fn consume_whitespace(&mut self) {
        while let Some(chr) = self.peek_char() {
            if chr.is_ascii_whitespace() {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        let chars: Vec<char> = self.input.chars().collect();
        if self.cursor < chars.len() {
            Some(chars[self.cursor])
        } else {
            None
        }
    }

    fn peek(&self) -> Option<ExprType> {
        let next = self.peek_char().unwrap_or(' ');

        if next == '(' {
            Some(ExprType::List)
        } else if next.is_ascii_alphabetic() {
            Some(ExprType::Atom)
        } else if next.is_ascii_digit() {
            Some(ExprType::Literal)
        } else {
            None
        }
    }

    fn parse_atom(&mut self) -> Option<Expr> {
        let start = self.cursor;
        while self.peek_char().unwrap_or(' ').is_ascii_alphabetic() {
            self.cursor += 1;
        }

        if start < self.cursor {
            let atom_string = self.input[start..self.cursor].to_string();
            Some(Expr(Box::new(ExprNode::Atom(atom_string))))
        } else {
            None
        }
    }

    fn parse_literal(&mut self) -> Option<Expr> {
        let start = self.cursor;
        while self.peek_char().unwrap_or(' ').is_ascii_digit() {
            self.cursor += 1;
        }

        let literal_string = self.input[start..self.cursor].to_string();
        let maybe_literal_value = literal_string.parse::<usize>().ok();
        maybe_literal_value.map(|literal_value|
            Expr(Box::new(ExprNode::Literal(literal_value)))
        )
    }

    fn parse_list(&mut self) -> Option<Expr> {
        if self.peek_char() == Some('(') {
            self.cursor += 1;
            self.consume_whitespace();
            let mut exprs = Vec::new();
            while self.peek_char() != Some(')') {
                let expr = self.parse();
                match expr {
                    None => return None,
                    Some(expr) => exprs.push(expr),
                }
            }
            self.cursor += 1; // consume ')'
            Some(Expr(Box::new(ExprNode::List(exprs))))
        } else {
            None
        }
    }

    fn parse(&mut self) -> Option<Expr> {
        self.consume_whitespace();
        match self.peek() {
            Some(ExprType::List) => self.parse_list(),
            Some(ExprType::Atom) => self.parse_atom(),
            Some(ExprType::Literal) => self.parse_literal(),
            None => None,
        }
    }
}
