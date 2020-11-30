use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;
use crate::ast;
use ast::HoleId;
use ast::HoleInfo;

type ParseErr = String;

struct Parser {
    tokens: Vec<Token>,
    cur: usize,
    next_hole_id: HoleId,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            cur: 0,
            next_hole_id: 0,
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
                Token::Hole(name, contents) => {
                    self.consume();
                    Ok(Some(ast::TermNode::Hole(HoleInfo::new(self.generate_hole_id(), name, contents)).into()))
                }
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn generate_hole_id(&mut self) -> HoleId {
        let hole_id = self.next_hole_id;
        self.next_hole_id += 1;
        hole_id
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

    fn parse_def(&mut self) -> Result<ast::Def, ParseErr> {
        self.consume_expect(Token::Def)?;
        let idents = self.consume_identifier_plus()?;
        let (binding_name, var_names) = idents.split_first().unwrap();
        self.consume_expect(Token::Equals)?;
        let mut body = self.parse_term()?;
        for var_name in var_names.into_iter().rev() {
            body = ast::TermNode::Lam(var_name.to_string(), body).into();
        }
        Ok(ast::Def(binding_name.to_string(), body))
    }

    fn parse_import(&mut self) -> Result<ast::Import, ParseErr> {
        self.consume_expect(Token::Import)?;
        let import_name = self.consume_identifier()?;
        Ok(ast::Import(import_name))
    }

    fn parse_module(&mut self) -> Result<ast::Module, ParseErr> {
        let mut definitions = Vec::new();
        let mut imports = Vec::new();

        while let Some(token) = self.peek() {
            if token == Token::Def {
                let definition = self.parse_def()?;
                definitions.push(definition );
            } else if token == Token::Import {
                let import = self.parse_import()?;
                imports.push(import);
            } else {
                return Err(format!("Expected an item declaration, found {:?}", token));
            }
        }
        Ok(ast::Module::new(definitions, imports))
    }
}

pub fn parse_term(input: &str) -> Result<ast::Term, ParseErr> {
    let mut toker = Tokenizer::new(input);
    let tokens = toker.tokenize();

    let mut parser = Parser::new(tokens);

    parser.parse_term()
}

pub fn parse_module(input: &str) -> Result<ast::Module, ParseErr> {
    let mut toker = Tokenizer::new(input);
    let tokens = toker.tokenize();

    let mut parser = Parser::new(tokens);

    parser.parse_module()
}
