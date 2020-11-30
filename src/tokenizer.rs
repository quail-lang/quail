use std::fmt;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Ident(Loc, String),
    Hole(Loc, Option<String>, Option<String>),
    Lambda(Loc),
    Let(Loc),
    Def(Loc),
    Equals(Loc),
    In(Loc),
    Arrow(Loc),
    FatArrow(Loc),
    LeftParen(Loc),
    RightParen(Loc),
    LeftCurly(Loc),
    RightCurly(Loc),
    Match(Loc),
    With(Loc),
    Import(Loc),
    Colon(Loc),
    As(Loc),
    Str(Loc, String),
}

pub struct Tokenizer {
    input: Vec<char>,
    cur: usize,
    loc: Loc,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Loc {
    pub path: Option<PathBuf>,

    // Both line and col are zero-based, so be mindful when printing!
    pub line: usize,
    pub col: usize,
}

type TokenizeErr = String;

impl Tokenizer {
    pub fn new(source: Option<&Path>, input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            cur: 0,
            loc: Loc::new(source),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizeErr> {
        let mut tokens = Vec::new();

        while let Some(token) = self.token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn token(&mut self) -> Result<Option<Token>, TokenizeErr> {
        while let Some(head_char) = self.peek() {
            if head_char.is_ascii_whitespace() {
                self.consume();
            } else if head_char == '#' {
                self.consume_comment();
            } else {
                break;
            }
        }

        let loc = self.loc.clone();
        match self.peek() {
            Some(head_char) => {
                if head_char == '(' {
                    self.consume();
                    Ok(Some(Token::LeftParen(loc)))
                } else if head_char == ')' {
                    self.consume();
                    Ok(Some(Token::RightParen(loc)))
                } else if head_char == '{' {
                    self.consume();
                    Ok(Some(Token::LeftCurly(loc)))
                } else if head_char == '}' {
                    self.consume();
                    Ok(Some(Token::RightCurly(loc)))
                } else if head_char.is_ascii_alphabetic() {
                    let token = self.tokenize_identifier()?;
                    Ok(Some(token))
                } else if head_char == ':' {
                    self.consume();
                    Ok(Some(Token::Colon(loc)))
                } else if head_char == '?' {
                    Ok(Some(self.tokenize_hole()?))
                } else if head_char == '"' {
                    Ok(Some(self.tokenize_str()?))
                } else if head_char == '-' {
                    match self.peek_ahead(1) {
                        Some('>') => {
                            self.consume();
                            self.consume();
                            Ok(Some(Token::Arrow(loc)))
                        },
                        _ => Err("Expected '>'".to_string()),
                    }
                } else if head_char == '=' {
                    match self.peek_ahead(1) {
                        Some('>') => {
                            self.consume();
                            self.consume();
                            Ok(Some(Token::FatArrow(loc)))
                        },
                        Some(_) => {
                            self.consume();
                            Ok(Some(Token::Equals(loc)))
                        }
                        None => Ok(Some(Token::Equals(loc))),
                    }
                } else {
                    Err(format!("Unexpected character while parsing: {}", head_char))
                }
            },
            None => Ok(None),
        }
    }

    fn tokenize_hole(&mut self) -> Result<Token, TokenizeErr> {
        let loc = self.loc.clone();
        assert_eq!(self.consume(), Some('?'));

        let peek_char : char;
        let name: Option<String>;

        match self.peek() {
            None => return Ok(Token::Hole(loc, None, None)),
            Some(chr) => peek_char = chr,
        }

        if peek_char.is_ascii_alphabetic() {
            if let Token::Ident(_, token_name) = self.tokenize_identifier()? {
                name = Some(token_name);
            } else {
                // TODO explain why
                unreachable!();
            }
        } else {
            name = None;
        }

        if let Some('{') = self.peek() {
            let mut level = 1;
            let mut contents = String::new();
            self.consume(); // Eat the '{'
            while let Some(peek_char) = self.consume() {
                if peek_char == '{' {
                    level += 1;
                } else if peek_char == '}' {
                    level -= 1;
                }

                if level == 0 {
                    break;
                } else {
                    contents.push(peek_char);
                }
            }

            if level != 0 {
                Err("Mismatch curly braces.".to_string())
            } else {
                Ok(Token::Hole(loc, name, Some(contents)))
            }
        } else {
            Ok(Token::Hole(loc, name, None))
        }
    }

    fn tokenize_str(&mut self) -> Result<Token, TokenizeErr> {
        #![allow(irrefutable_let_patterns)]
        let loc = self.loc.clone();
        assert_eq!(self.consume(), Some('"'));

        let mut buffer = String::new();

        while let consume_char = self.consume() {
            match consume_char {
                None => return Err("Expected \" but found end of file. Good luck!".to_string()),
                Some(chr) => {
                    if chr == '"' {
                        break;
                    } else {
                        buffer.push(chr);
                    }
                },
            }
        }

        Ok(Token::Str(loc, buffer))
    }

    fn tokenize_identifier(&mut self) -> Result<Token, TokenizeErr> {
        let keywords: HashMap<String, Token> = vec![
            ("fun".to_string(), Token::Lambda(self.loc.clone())),
            ("let".to_string(), Token::Let(self.loc.clone())),
            ("def".to_string(), Token::Def(self.loc.clone())),
            ("in".to_string(), Token::In(self.loc.clone())),
            ("match".to_string(), Token::Match(self.loc.clone())),
            ("with".to_string(), Token::With(self.loc.clone())),
            ("import".to_string(), Token::Import(self.loc.clone())),
            ("as".to_string(), Token::As(self.loc.clone())),
        ].iter().cloned().collect();

        let loc = self.loc.clone();

        let mut first_char = '\0';
        match self.peek() {
            Some(chr) => {
                self.consume();
                first_char = chr;
            },
            None => assert!(first_char.is_ascii_alphabetic()),
        }

        let mut token_string = String::new();
        token_string.push(first_char);

        while let Some(peek_char) = self.peek() {
            if peek_char.is_ascii_alphabetic() || peek_char == '_' {
                self.consume();
                token_string.push(peek_char);
            } else {
                break;
            }
        }

        // Allow primes ' at the end of identifiers.
        while let Some(peek_char) = self.peek() {
            if peek_char == '\'' {
                self.consume();
                token_string.push(peek_char);
            } else {
                break;
            }
        }

        match keywords.get(&token_string) {
            Some(token) => Ok(token.clone()),
            None => Ok(Token::Ident(loc, token_string))
        }
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
                if peek_char == '\n' {
                    self.loc.next_line();
                } else {
                    self.loc.next_col();
                }
                self.cur += 1;
                Some(peek_char)
            },
            None => None,
        }
    }

    fn consume_comment(&mut self) {
        while let Some(consume_char) = self.consume() {
            if consume_char == '\n' {
                break
            }
        }
    }
}

impl Loc {
    fn new(source: Option<&Path>) -> Self {
        Loc {
            path: source.map(|p| p.to_path_buf()),
            line: 0,
            col: 0,
        }
    }

    fn next_line(&mut self) {
        self.line += 1;
        self.col = 0;
    }

    fn next_col(&mut self) {
        self.col += 1;
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {} col {}", self.line + 1, self.col + 1)?;
        if let Some(path) = &self.path {
            write!(f, " at {}", path.to_string_lossy())?;
        }
        Ok(())
    }
}
