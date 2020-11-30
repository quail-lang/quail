use crate::ast;
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

type ParseErr = String;

struct Parser {
    tokens: Vec<Token>,
    cur: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            cur: 0,
        }
    }

    fn peek(&mut self) -> Option<Token> {
        self.peek_ahead(0)
    }

    fn peek_ahead(&mut self, k: usize) -> Option<Token> {
        match self.tokens.get(self.cur + k) {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }

    fn consume(&mut self) -> Option<Token> {
        match self.peek() {
            Some(peek_token) => {
                self.cur += 1;
                Some(peek_token)
            },
            None => None,
        }
    }

    fn consume_expect(&mut self, expected_token: Token) -> Result<(), ParseErr> {
        match self.peek() {
            Some(peek_token) => {
                if peek_token == expected_token {
                    self.cur += 1;
                    Ok(())
                } else {
                    Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token))
                }
            },
            None => {
                Err(format!("Expected {:?} but found end of input.", expected_token))
            },
        }
    }

    fn consume_identifier(&mut self) -> Result<String, ParseErr> {
        match self.consume() {
            Some(Token::Ident(name)) => Ok(name),
            Some(token) => Err(format!("Expected identifier but found {:?}.", token)),
            None => Err("Expected identifier but found end of input.".into()),
        }
    }

    fn parse_lambda(&mut self) -> Result<ast::Term, ParseErr> {
        self.consume_expect(Token::Lambda)?;
        let bind_var = self.consume_identifier()?;
        self.consume_expect(Token::FatArrow)?;
        let body = self.parse_term()?;
        Ok(ast::TermNode::Lam(bind_var, body).into())
    }

    fn parse_term_part(&mut self) -> Result<Option<ast::Term>, ParseErr> {
        match self.peek() {
            Some(token) => match token {
                Token::Ident(name) => {
                    self.consume();
                    Ok(Some(ast::TermNode::Var(name).into()))
                },
                Token::Lit(value) => {
                    self.consume();
                    Ok(Some(ast::TermNode::NatLit(value).into()))
                },
                Token::Lambda => Ok(Some(self.parse_lambda()?)),
                Token::FatArrow => Err("Can't start a term with a fat array ^^;;".into()),
                Token::LeftParen => {
                    self.consume_expect(Token::LeftParen)?;
                    let term = self.parse_term();
                    self.consume_expect(Token::RightParen)?;
                    Ok(Some(term?))
                }
                Token::RightParen => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn parse_term(&mut self) -> Result<ast::Term, ParseErr> {
        let mut term;
        match self.parse_term_part()? {
            None => {
                return Err("Empty input".to_string());
            },
            Some(term_part) => {
                term = term_part;
            },
        }

        let mut term_parts: Vec<ast::Term> = Vec::new();
        while let Some(term_part) = self.parse_term_part()? {
            term_parts.push(term_part);
        }

        for term_part in term_parts.into_iter() {
            term = ast::TermNode::App(term, term_part).into();
        }
        Ok(term)
    }

    fn parse(&mut self) -> Result<ast::Term, ParseErr> {
        let term = self.parse_term();
        match self.peek() {
            None => term,
            Some(token) => Err(format!("Unexpected {:?} token at end of stream", token)),
        }
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;

    #[test]
    fn test_a() {
        let identity_fn: ast::Term = ast::TermNode::Lam(
            "x".into(),
            ast::TermNode::Var("x".into()).into(),
        ).into();
        assert_eq!(
            parse("fn x => x"),
            Ok(identity_fn),
        );
    }
}

pub fn parse(input: impl Into<String>) -> Result<ast::Term, ParseErr> {
    let mut toker = Tokenizer::new(input);
    let tokens = toker.tokenize();
    println!("{:?}", &tokens);

    let mut parser = Parser::new(tokens);

    parser.parse()
}
