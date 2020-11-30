use crate::ast;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Ident(String),
    Lit(u64),
    Lambda,
    FatArrow,
    LeftParen,
    RightParen,
}


pub struct Tokenizer {
    input: Vec<char>,
    cur: usize,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tokenize_ident_then_lit() {
        let mut toker = Tokenizer::new("abcd 123");
        assert_eq!(
            toker.tokenize(),
            vec![Token::Ident("abcd".to_string()), Token::Lit(123)]
        );
    }

    #[test]
    fn tokenize_empty_string() {
        let mut toker = Tokenizer::new("");
        assert_eq!(
            toker.tokenize(),
            vec![]
        );
    }

    #[test]
    fn tokenize_test_a() {
        let mut toker = Tokenizer::new("(a)");
        assert_eq!(
            toker.tokenize(),
            vec![Token::LeftParen, Token::Ident("a".to_string()), Token::RightParen]
        );
    }

    #[test]
    fn tokenize_test_b() {
        let mut toker = Tokenizer::new("fn x => x");
        assert_eq!(
            toker.tokenize(),
            vec![Token::Lambda, Token::Ident("x".to_string()), Token::FatArrow, Token::Ident("x".to_string())]
        );
    }
}

impl Tokenizer {

    pub(crate) fn new(input: impl Into<String>) -> Self {
        Tokenizer {
            input: input.into().chars().collect(),
            cur: 0,
        }
    }

    pub(crate) fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(head_char) = self.peek() {
            if head_char == ' ' {
                self.consume();
            } else if head_char == '(' {
                tokens.push(Token::LeftParen);
                self.consume();
            } else if head_char == ')' {
                tokens.push(Token::RightParen);
                self.consume();
            } else if head_char.is_ascii_digit() {
                let token = self.tokenize_literal();
                tokens.push(token);
            } else if head_char.is_ascii_alphabetic() {
                let token = self.tokenize_identifier();
                tokens.push(token);
            } else if head_char == '=' {
                match self.peek_ahead(1) {
                    Some('>') => {
                        tokens.push(Token::FatArrow);
                        self.consume();
                        self.consume();
                    },
                    Some(_) => panic!("Uh oh #1"),
                    None => panic!("Uh oh #2"),
                }
            } else {
                panic!("Uh oh #3");
            }
        }
        tokens
    }

    fn peek(&mut self) -> Option<char> {
        self.peek_ahead(0)
    }

    fn peek_ahead(&mut self, k: usize) -> Option<char> {
        match self.input.get(self.cur + k) {
            Some(c) => Some(*c),
            None => None,
        }
    }

    fn consume(&mut self) -> Option<char> {
        match self.peek() {
            Some(peek_char) => {
                self.cur += 1;
                Some(peek_char)
            },
            None => None,
        }
    }

    fn tokenize_literal(&mut self) -> Token {
        let first_digit = self.input[self.cur];
        assert!(first_digit.is_ascii_digit());
        let mut token_string = String::new();
        token_string.push(first_digit);

        let mut new_cur = self.cur + 1;
        while new_cur < self.input.len() && self.input[new_cur].is_ascii_digit() {
            token_string.push(self.input[new_cur]);
            new_cur += 1;
        }

        self.cur = new_cur;
        return Token::Lit(token_string.parse::<u64>().expect("Should be valid integer."));
    }

    fn tokenize_identifier(&mut self) -> Token {
        let keywords: HashMap<String, Token> = vec![
            ("fn".to_string(), Token::Lambda),
        ].iter().cloned().collect();
        let first_char = self.input[self.cur];
        assert!(first_char.is_ascii_alphabetic());
        let mut token_string = String::new();
        token_string.push(first_char);

        let mut new_cur = self.cur + 1;
        while new_cur < self.input.len() && self.input[new_cur].is_ascii_alphabetic() {
            token_string.push(self.input[new_cur]);
            new_cur += 1;
        }

        self.cur = new_cur;
        match keywords.get(&token_string) {
            Some(token) => token.clone(),
            None => Token::Ident(token_string)
        }
    }
}

pub fn parse(input: impl Into<String>) -> ast::Term {
    let expr = parse_sexpr(input).expect("Could not parse.");
    expr_to_term(expr)
}

fn assert_atom(expr: Expr) -> String {
    let Expr(expr_node) = expr;
    match *expr_node {
        ExprNode::Atom(s) => s,
        _ => panic!("assertion failed: not an Atom."),
    }
}

fn expr_to_term(expr: Expr) -> ast::Term {
    let Expr(expr_node) = expr;
    match *expr_node {
        ExprNode::List(exprs) => {
            let Expr(head_expr_node) = exprs[0].clone();
            match *head_expr_node {
                ExprNode::Atom(v) => {
                    match v.as_ref() {
                        "fn" => {
                            ast::TermNode::Lam(
                                assert_atom(exprs[1].clone()),
                                ast::Type::Bool,
                                expr_to_term(exprs[2].clone()),
                            ).into()
                        },
                        vs => {
                            let x = expr_to_term(exprs[1].clone());
                            ast::TermNode::App(
                                ast::TermNode::Var(vs.to_string()).into(),
                                x,
                            ).into()
                        },
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
