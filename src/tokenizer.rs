use std::collections::HashMap;

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
}

impl Tokenizer {

    pub(crate) fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
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
        ].into_iter().collect();

        while let Some(head_char) = self.peek() {
            if head_char.is_ascii_whitespace() {
                self.consume();
            } else if single_char_tokens.contains_key(&head_char) {
                let token = single_char_tokens.get(&head_char).unwrap().clone();
                tokens.push(token);
                self.consume();
            } else if head_char.is_ascii_alphabetic() {
                let token = self.tokenize_identifier();
                tokens.push(token);
            } else if head_char == '#' {
                self.consume_comment();
            } else if head_char == '?' {
                tokens.push(self.tokenize_hole());
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

    fn consume_comment(&mut self) {
        while let Some(consume_char) = self.consume() {
            if consume_char == '\n' {
                break
            }
        }
    }

    fn tokenize_hole(&mut self) -> Token {
        assert_eq!(self.consume(), Some('?'));

        let peek_char : char;
        let name: Option<String>;

        match self.peek() {
            None => return Token::Hole(None, None),
            Some(chr) => peek_char = chr,
        }

        if peek_char.is_ascii_alphabetic() {
            if let Token::Ident(token_name) = self.tokenize_identifier() {
                name = Some(token_name);
            } else {
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
                panic!("Mismatch curly braces.")
            } else {
                Token::Hole(name, Some(contents))
            }
        } else {
            Token::Hole(name, None)
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

    /*
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
    */

    fn tokenize_identifier(&mut self) -> Token {
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

        self.cur = new_cur;
        match keywords.get(&token_string) {
            Some(token) => token.clone(),
            None => Token::Ident(token_string)
        }
    }
}
