use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Ident(String),
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
    Hole(Option<String>, Option<String>),
    Match,
    With,
    Import,
}

pub struct Tokenizer {
    input: Vec<char>,
    cur: usize,
    loc: Location,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub path: Option<PathBuf>,

    // Both line and col are zero-based, so be mindful when printing!
    pub line: usize,
    pub col: usize,
}

type TokenizeErr = String;

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            cur: 0,
            loc: Location::new(None),
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
        let single_char_tokens: HashMap<char, Token> = vec![
            ('(', Token::LeftParen),
            (')', Token::RightParen),
            ('[', Token::LeftSquare),
            (']', Token::RightSquare),
            ('{', Token::LeftCurly),
            ('}', Token::RightCurly),
        ].into_iter().collect();

        while let Some(head_char) = self.peek() {
            if head_char.is_ascii_whitespace() {
                self.consume();
            } else if head_char == '#' {
                self.consume_comment();
            } else {
                break;
            }
        }

        match self.peek() {
            Some(head_char) => {
                if single_char_tokens.contains_key(&head_char) {
                    let token = single_char_tokens.get(&head_char).unwrap().clone();
                    self.consume();
                    Ok(Some(token))
                } else if head_char.is_ascii_alphabetic() {
                    let token = self.tokenize_identifier()?;
                    Ok(Some(token))
                } else if head_char == '?' {
                    Ok(Some(self.tokenize_hole()?))
                } else if head_char == '=' {
                    match self.peek_ahead(1) {
                        Some('>') => {
                            self.consume();
                            self.consume();
                            Ok(Some(Token::FatArrow))
                        },
                        Some(_) => {
                            self.consume();
                            Ok(Some(Token::Equals))
                        }
                        None => Ok(Some(Token::Equals)),
                    }
                } else {
                    Err(format!("Unexpected character while parsing: {}", head_char))
                }
            },
            None => Ok(None),
        }
    }

    fn consume_comment(&mut self) {
        while let Some(consume_char) = self.consume() {
            if consume_char == '\n' {
                break
            }
        }
    }

    fn tokenize_hole(&mut self) -> Result<Token, TokenizeErr> {
        assert_eq!(self.consume(), Some('?'));

        let peek_char : char;
        let name: Option<String>;

        match self.peek() {
            None => return Ok(Token::Hole(None, None)),
            Some(chr) => peek_char = chr,
        }

        if peek_char.is_ascii_alphabetic() {
            if let Token::Ident(token_name) = self.tokenize_identifier()? {
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
                Ok(Token::Hole(name, Some(contents)))
            }
        } else {
            Ok(Token::Hole(name, None))
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
                self.cur += 1;
                Some(peek_char)
            },
            None => None,
        }
    }

    fn tokenize_identifier(&mut self) -> Result<Token, TokenizeErr> {
        let keywords: HashMap<String, Token> = vec![
            ("fun".to_string(), Token::Lambda),
            ("let".to_string(), Token::Let),
            ("def".to_string(), Token::Def),
            ("in".to_string(), Token::In),
            ("match".to_string(), Token::Match),
            ("with".to_string(), Token::With),
            ("import".to_string(), Token::Import),
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

        // TODO new_cur is really bad because it overwrites self.cur
        // without respect to the position information.
        self.cur = new_cur;
        match keywords.get(&token_string) {
            Some(token) => Ok(token.clone()),
            None => Ok(Token::Ident(token_string))
        }
    }
}

impl Location {
    fn new(source: Option<&Path>) -> Self {
        Location {
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
