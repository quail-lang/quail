use std::fmt;
use std::collections::HashMap;

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
    Dollar(Loc),
    As(Loc),
    Str(Loc, String),
    Nat(Loc, usize),
}

pub struct Tokenizer {
    input: Vec<char>,
    cur: usize,
    loc: Loc,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Loc {
    pub path: Option<String>,

    // Both line and col are zero-based, so be mindful when printing!
    pub line: usize,
    pub col: usize,
}

type TokenizeErr = String;

impl Token {
    pub fn name(&self) -> &'static str {
        use Token::*;
        match self {
            Ident(_loc, _x) => "IDENT",
            Hole(_loc, _x, _contents) => "HOLE",
            Lambda(_loc) => "LAMBDA",
            Let(_loc) => "LET",
            Def(_loc) => "DEF",
            Equals(_loc) => "EQUALS",
            In(_loc) => "IN",
            Arrow(_loc) => "ARROW",
            FatArrow(_loc) => "FATARROW",
            LeftParen(_loc) => "LEFTPAREN",
            RightParen(_loc) => "RIGHTPAREN",
            LeftCurly(_loc) => "LEFTCURLY",
            RightCurly(_loc) => "RIGHTCURLY",
            Match(_loc) => "MATCH",
            With(_loc) => "WITH",
            Import(_loc) => "IMPORT",
            Colon(_loc) => "COLON",
            Dollar(_loc) => "DOLLAR",
            As(_loc) => "AS",
            Str(_loc, _val) => "STR",
            Nat(_loc, _val) => "NAT",
        }
    }

    pub fn show(&self) -> String {
        use Token::*;
        match self {
            Ident(_loc, x) => format!("IDENT({})", x),
            Hole(_loc, x, _contents) => format!("HOLE({:?}, ...)", x),
            Lambda(_loc) => format!("LAMBDA"),
            Let(_loc) => format!("LET"),
            Def(_loc) => format!("DEF"),
            Equals(_loc) => format!("EQUALS"),
            In(_loc) => format!("IN"),
            Arrow(_loc) => format!("ARROW"),
            FatArrow(_loc) => format!("FATARROW"),
            LeftParen(_loc) => format!("LEFTPAREN"),
            RightParen(_loc) => format!("RIGHTPAREN"),
            LeftCurly(_loc) => format!("LEFTCURLY"),
            RightCurly(_loc) => format!("RIGHTCURLY"),
            Match(_loc) => format!("MATCH"),
            With(_loc) => format!("WITH"),
            Import(_loc) => format!("IMPORT"),
            Colon(_loc) => format!("COLON"),
            Dollar(_loc) => format!("DOLLAR"),
            As(_loc) => format!("AS"),
            Str(_loc, val) => format!("STR({})", val),
            Nat(_loc, val) => format!("NAT({})", val),
        }
    }

    pub fn loc(&self) -> &Loc {
        use Token::*;
        match self {
            Ident(loc, _x) => loc,
            Hole(loc, _x, _contents) => loc,
            Lambda(loc) => loc,
            Let(loc) => loc,
            Def(loc) => loc,
            Equals(loc) => loc,
            In(loc) => loc,
            Arrow(loc) => loc,
            FatArrow(loc) => loc,
            LeftParen(loc) => loc,
            RightParen(loc) => loc,
            LeftCurly(loc) => loc,
            RightCurly(loc) => loc,
            Match(loc) => loc,
            With(loc) => loc,
            Import(loc) => loc,
            Colon(loc) => loc,
            Dollar(loc) => loc,
            As(loc) => loc,
            Str(loc, _val) => loc,
            Nat(loc, _val) => loc,
        }
    }
}

impl Tokenizer {
    pub fn new(source: Option<String>, input: &str) -> Self {
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

    fn tokenize_lines(&mut self) -> Result<Vec<Vec<Token>>, TokenizeErr> {
        let toks: Vec<Token> = self.tokenize()?;

        let mut lines: Vec<Vec<Token>> = Vec::new();
        let mut cur_line: Vec<Token> = Vec::new();

        let mut line_no = 0;

        for tok in toks {
            while tok.loc().line > line_no {
                line_no += 1;
                lines.push(cur_line);
                cur_line = Vec::new();
            }
            cur_line.push(tok);
        }
        lines.push(cur_line);

        Ok(lines)
    }

    fn double_character_token(&mut self) -> Option<Token> {
        let head_char = self.peek()?;
        let next_char = self.peek_ahead(1)?;
        let chars = format!("{}{}", head_char, next_char);

        macro_rules! double_char_token {
            ($characters:literal, $tok:ident) => {
                if chars == $characters {
                    self.consume();
                    self.consume();
                    return Some(Token::$tok(self.loc.clone()));
                }
            }
        }

        double_char_token!("->", Arrow);
        double_char_token!("=>", FatArrow);

        return None;
    }

    fn single_character_token(&mut self) -> Option<Token> {
        let head_char = self.peek()?;

        macro_rules! single_char_token {
            ($character:literal, $tok:ident) => {
                if head_char == $character {
                    self.consume();
                    return Some(Token::$tok(self.loc.clone()));
                }
            }
        }

        single_char_token!('(', LeftParen);
        single_char_token!(')', RightParen);
        single_char_token!('{', LeftCurly);
        single_char_token!('}', RightCurly);
        single_char_token!(':', Colon);
        single_char_token!('$', Dollar);
        single_char_token!('=', Equals);

        return None;
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

        match self.peek() {
            Some(head_char) => {
                if let Some(tok) = self.double_character_token() {
                    Ok(Some(tok))
                } else if let Some(tok) = self.single_character_token() {
                    Ok(Some(tok))
                } else if head_char.is_ascii_alphabetic() {
                    let token = self.tokenize_identifier()?;
                    Ok(Some(token))
                } else if head_char == '?' {
                    Ok(Some(self.tokenize_hole()?))
                } else if head_char == '"' {
                    Ok(Some(self.tokenize_str()?))
                } else if head_char.is_ascii_digit() {
                    Ok(Some(self.tokenize_nat()?))
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

    fn tokenize_nat(&mut self) -> Result<Token, TokenizeErr> {
        let loc = self.loc.clone();
        let mut buffer = String::new();
        match self.peek() {
            None => return Err("Expected digit but found end of file. Good luck!".to_owned()),
            Some(ch) => {
                if !ch.is_ascii_digit() {
                    return Err(format!("Expected digit but found {}.", ch));
                }

                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        self.consume();
                        buffer.push(ch);

                    } else {
                        break;
                    }
                }
            }
        }
        let n = buffer.parse::<usize>().unwrap();
        Ok(Token::Nat(loc, n))
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

    fn peek(&self) -> Option<char> {
        self.peek_ahead(0)
    }

    fn peek_ahead(&self, k: usize) -> Option<char> {
        match self.input.get(self.cur + k) {
            Some(c) => Some(*c),
            None => None,
        }
    }

    #[allow(dead_code)]
    fn preview(&self, len: usize) -> String {
        let mut s = String::new();
        for i in 0..len {
            if let Some(ch) = self.peek_ahead(i) {
                s.push(ch);
            } else {
                break;
            }
        }
        s
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
    fn new(source: Option<String>) -> Self {
        Loc {
            path: source,
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
            write!(f, " at {}", path)?;
        }
        Ok(())
    }
}

pub fn tokenize(source: Option<String>, input: &str) -> Result<Vec<Token>, TokenizeErr> {
    let mut tokenizer = Tokenizer::new(source, input);
    tokenizer.tokenize()
}

pub fn tokenize_lines(source: Option<String>, input: &str) -> Result<Vec<Vec<Token>>, TokenizeErr> {
    let mut tokenizer = Tokenizer::new(source, input);
    tokenizer.tokenize_lines()
}
