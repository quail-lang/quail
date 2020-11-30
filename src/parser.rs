use crate::ast;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Ident(String),
    Lit(u64),
    Lambda,
    Let,
    Def,
    Equals,
    In,
    FatArrow,
    LeftParen,
    RightParen,
    LeftCurly,
    RightCurly,
    LeftSquare,
    RightSquare,
    Question,
    Match,
    With,
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
        let mut toker = Tokenizer::new("fun x => x");
        assert_eq!(
            toker.tokenize(),
            vec![Token::Lambda, Token::Ident("x".to_string()), Token::FatArrow, Token::Ident("x".to_string())]
        );
    }

    #[test]
    fn tokenize_test_c() {
        let mut toker = Tokenizer::new("let f = succ in f 2");
        assert_eq!(
            toker.tokenize(),
            vec![
                Token::Let,
                Token::Ident("f".to_string()),
                Token::Equals,
                Token::Ident("succ".to_string()),
                Token::In,
                Token::Ident("f".to_string()),
                Token::Lit(2),
            ]
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

        let single_char_tokens: HashMap<char, Token> = vec![
            ('(', Token::LeftParen),
            (')', Token::RightParen),
            ('[', Token::LeftSquare),
            (']', Token::RightSquare),
            ('{', Token::LeftCurly),
            ('}', Token::RightCurly),
            ('?', Token::Question),
        ].into_iter().collect();

        while let Some(head_char) = self.peek() {
            if head_char.is_ascii_whitespace() {
                self.consume();
            } else if single_char_tokens.contains_key(&head_char) {
                let token = single_char_tokens.get(&head_char).unwrap().clone();
                tokens.push(token);
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
                    Some(_) => {
                        tokens.push(Token::Equals);
                        self.consume();
                    }
                    None => tokens.push(Token::Equals),
                }
            } else {
                panic!("Unexpected character while parsing: {}", head_char);
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
            ("fun".to_string(), Token::Lambda),
            ("let".to_string(), Token::Let),
            ("def".to_string(), Token::Def),
            ("in".to_string(), Token::In),
            ("match".to_string(), Token::Match),
            ("with".to_string(), Token::With),
        ].iter().cloned().collect();
        let first_char = self.input[self.cur];
        assert!(first_char.is_ascii_alphabetic());
        let mut token_string = String::new();
        token_string.push(first_char);

        let mut new_cur = self.cur + 1;
        while new_cur < self.input.len() &&
            (self.input[new_cur].is_ascii_alphabetic() || self.input[new_cur] == '_') {
            token_string.push(self.input[new_cur]);
            new_cur += 1;
        }

        // Allow primes ' at the end of identifiers.
        while new_cur < self.input.len() && self.input[new_cur] == '\'' {
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

    fn consume_identifier_plus(&mut self) -> Result<Vec<String>, ParseErr> {
        let mut idents = Vec::new();
        while let Some(token) = self.peek() {
            if let Token::Ident(name) = token {
                idents.push(name);
                self.consume();
            } else {
                break;
            }
        }
        if idents.is_empty() {
            Err("Expected identifier".into())
        } else {
            Ok(idents)
        }
    }

    fn parse_lambda(&mut self) -> Result<ast::Term, ParseErr> {
        self.consume_expect(Token::Lambda)?;
        let bind_vars = self.consume_identifier_plus()?;
        self.consume_expect(Token::FatArrow)?;
        let body = self.parse_term()?;

        let mut term = body;
        for bind_var in bind_vars.into_iter().rev() {
            term = ast::TermNode::Lam(bind_var, term).into();
        }
        Ok(term)
    }

    fn parse_hole(&mut self) -> Result<Option<ast::Term>, ParseErr> {
        self.consume_expect(Token::Question)?;
        match self.peek() {
            Some(Token::LeftCurly) => {
                let mut level = 0;
                while let Some(token) = self.consume() {
                    if token == Token::LeftCurly {
                        level += 1;
                    } else if token == Token::RightCurly {
                        level -= 1;
                    }
                    if level == 0 {
                        break;
                    }
                }

                if level != 0 {
                    Err("Unclosed { when parsing a hole.".to_string())
                } else {
                    Ok(Some(ast::TermNode::Hole.into()))
                }
            }
            _ => Ok(Some(ast::TermNode::Hole.into())),
        }
    }

    fn parse_term_part(&mut self) -> Result<Option<ast::Term>, ParseErr> {
        match self.peek() {
            Some(token) => match token {
                Token::Ident(name) => {
                    self.consume();
                    Ok(Some(ast::TermNode::Var(name).into()))
                },
                Token::Lambda => Ok(Some(self.parse_lambda()?)),
                Token::LeftParen => {
                    self.consume_expect(Token::LeftParen)?;
                    let term = self.parse_term();
                    self.consume_expect(Token::RightParen)?;
                    Ok(Some(term?))
                }
                Token::RightParen => Ok(None),
                Token::Let => {
                    self.consume_expect(Token::Let)?;
                    let bind_var = self.consume_identifier()?;
                    self.consume_expect(Token::Equals)?;
                    let value = self.parse_term()?;
                    self.consume_expect(Token::In)?;
                    let body = self.parse_term()?;
                    Ok(Some(ast::TermNode::Let(bind_var, value, body).into()))
                },
                Token::Question => Ok(self.parse_hole()?),
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn parse_pattern(&mut self) -> Result<ast::Pattern, ParseErr> {
        Ok(self.consume_identifier_plus()?)
    }

    fn parse_match_arm(&mut self) -> Result<ast::MatchArm, ParseErr> {
        self.consume_expect(Token::With)?;
        let idents = self.parse_pattern()?;
        self.consume_expect(Token::FatArrow)?;
        let body = self.parse_term()?;
        Ok(ast::MatchArm(idents, body))
    }

    fn parse_match_arm_plus(&mut self) -> Result<Vec<ast::MatchArm>, ParseErr> {
        let mut match_arms = Vec::new();
        match_arms.push(self.parse_match_arm()?);
        while let Some(Token::With) = self.peek() {
            match_arms.push(self.parse_match_arm()?);
        }
        Ok(match_arms)
    }

    fn parse_match(&mut self) -> Result<ast::Term, ParseErr> {
        self.consume_expect(Token::Match)?;
        let discriminee = self.parse_term()?;
        let match_arms = self.parse_match_arm_plus()?;
        Ok(ast::TermNode::Match(discriminee, match_arms).into())
    }

    fn parse_term(&mut self) -> Result<ast::Term, ParseErr> {
        if self.peek() == Some(Token::Match) {
            self.parse_match()
        } else {
            let func;
            let mut args = Vec::new();

            match self.parse_term_part()? {
                None => {
                    return Err("Empty input".to_string());
                },
                Some(term_part) => {
                    func = term_part;
                },
            }

            while let Some(term_part) = self.parse_term_part()? {
                args.push(term_part);
            }

            if args.is_empty() {
                Ok(func)
            } else {
                Ok(ast::TermNode::App(func, args).into())
            }
        }
    }

    fn parse_def(&mut self) -> Result<ast:: Item, ParseErr> {
        self.consume_expect(Token::Def)?;
        let binding_name = self.consume_identifier()?;
        self.consume_expect(Token::Equals)?;
        let body = self.parse_term()?;
        Ok(ast::Item::Def(binding_name, body))
    }

    fn parse_program(&mut self) -> Result<ast:: Program, ParseErr> {
        let mut items = Vec::new();
        while let Some(token) = self.peek() {
            if token == Token::Def {
                let item = self.parse_def()?;
                items.push(item);
            } else {
                return Err(format!("Expected an item declaration, found {:?}", token));
            }
        }
        Ok(ast::Program { items })
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
            parse_term("fun x => x"),
            Ok(identity_fn),
        );
    }
}

pub fn parse_term(input: impl Into<String>) -> Result<ast::Term, ParseErr> {
    let mut toker = Tokenizer::new(input);
    let tokens = toker.tokenize();

    let mut parser = Parser::new(tokens);

    parser.parse_term()
}

pub fn parse_program(input: impl Into<String>) -> Result<ast::Program, ParseErr> {
    let mut toker = Tokenizer::new(input);
    let tokens = toker.tokenize();

    let mut parser = Parser::new(tokens);

    parser.parse_program()
}
