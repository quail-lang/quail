use std::path::Path;

use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;
use crate::types::Type;
use crate::types::TypeNode;
use crate::ast;

use ast::HoleId;
use ast::HoleInfo;
use ast::Term;
use ast::TermNode;
use ast::Module;
use ast::MatchArm;
use ast::Def;
use ast::Import;
use ast::Pattern;
use ast::Variable;

type ParseErr = String;

struct Parser {
    tokens: Vec<Token>,
    cur: usize,
    next_hole_id: HoleId,
    hole_count: u64,
}

impl Parser {
    pub fn new(starting_hole_id: HoleId, tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            cur: 0,
            next_hole_id: starting_hole_id,
            hole_count: 0,
        }
    }

    pub fn number_of_holes(&self) -> u64 {
        self.hole_count
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

    fn consume_expect_lambda(&mut self) -> Result<(), ParseErr> {
        let expected_token = "fun";
        match self.peek() {
            Some(Token::Lambda(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_fat_arrow(&mut self) -> Result<(), ParseErr> {
        let expected_token = "=>";
        match self.peek() {
            Some(Token::FatArrow(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_arrow(&mut self) -> Result<(), ParseErr> {
        let expected_token = "=>";
        match self.peek() {
            Some(Token::Arrow(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_left_paren(&mut self) -> Result<(), ParseErr> {
        let expected_token = "(";
        match self.peek() {
            Some(Token::LeftParen(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }
    fn consume_expect_right_paren(&mut self) -> Result<(), ParseErr> {
        let expected_token = ")";
        match self.peek() {
            Some(Token::RightParen(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_let(&mut self) -> Result<(), ParseErr> {
        let expected_token = "let";
        match self.peek() {
            Some(Token::Let(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_equals(&mut self) -> Result<(), ParseErr> {
        let expected_token = "=";
        match self.peek() {
            Some(Token::Equals(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }
    fn consume_expect_colon(&mut self) -> Result<(), ParseErr> {
        let expected_token = ":";
        match self.peek() {
            Some(Token::Colon(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_match(&mut self) -> Result<(), ParseErr> {
        let expected_token = "match";
        match self.peek() {
            Some(Token::Match(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_with(&mut self) -> Result<(), ParseErr> {
        let expected_token = "with";
        match self.peek() {
            Some(Token::With(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_def(&mut self) -> Result<(), ParseErr> {
        let expected_token = "def";
        match self.peek() {
            Some(Token::Def(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_in(&mut self) -> Result<(), ParseErr> {
        let expected_token = "in";
        match self.peek() {
            Some(Token::In(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_import(&mut self) -> Result<(), ParseErr> {
        let expected_token = "import";
        match self.peek() {
            Some(Token::Import(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_expect_as(&mut self) -> Result<(), ParseErr> {
        let expected_token = "as";
        match self.peek() {
            Some(Token::As(_)) => {
                self.consume();
                Ok(())
            },
            Some(peek_token) => Err(format!("Expected {:?} but found {:?}.", expected_token, peek_token)),
            None => Err(format!("Expected {:?} but found end of input.", expected_token)),
        }
    }

    fn consume_identifier(&mut self) -> Result<String, ParseErr> {
        match self.consume() {
            Some(Token::Ident(_, name)) => Ok(name),
            Some(token) => Err(format!("Expected identifier but found {:?}.", token)),
            None => Err("Expected identifier but found end of input.".into()),
        }
    }

    fn consume_identifier_plus(&mut self) -> Result<Vec<String>, ParseErr> {
        let mut idents = Vec::new();
        while let Some(token) = self.peek() {
            if let Token::Ident(_, name) = token {
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

    fn parse_lambda(&mut self) -> Result<Term, ParseErr> {
        self.consume_expect_lambda()?;
        let bind_vars = self.consume_identifier_plus()?;
        self.consume_expect_fat_arrow()?;
        let body = self.parse_term()?;

        let mut term = body;
        for bind_var in bind_vars.into_iter().rev() {
            term = TermNode::Lam(bind_var, term).into();
        }
        Ok(term)
    }

    fn parse_variable(&mut self) -> Result<Term, ParseErr> {
        let name = self.consume_identifier()?;
        if let Some(Token::Dollar(_)) = self.peek() {
            if let Some(Token::Nat(_, k)) = self.peek_ahead(1) {
                self.consume();
                self.consume();
                let variable = Variable {
                    name,
                    layer: k,
                };
                Ok(TermNode::Var(variable).into())
            } else {
                Err("Expected a number after $.".to_string())
            }
        } else {
            let variable = Variable {
                name,
                layer: 0,
            };
            Ok(TermNode::Var(variable).into())
        }
    }

    fn parse_term_part(&mut self) -> Result<Option<Term>, ParseErr> {
        match self.peek() {
            Some(token) => match token {
                Token::Ident(_, _name) => {
                    Ok(Some(self.parse_variable()?))
                },
                Token::Lambda(_) => Ok(Some(self.parse_lambda()?)),
                Token::LeftParen(_) => {
                    self.consume_expect_left_paren()?;
                    let term = self.parse_term();
                    self.consume_expect_right_paren()?;
                    Ok(Some(term?))
                }
                Token::RightParen(_) => Ok(None),
                Token::Let(_) => {
                    self.consume_expect_let()?;
                    let bind_var = self.consume_identifier()?;
                    self.consume_expect_equals()?;
                    let value = self.parse_term()?;
                    self.consume_expect_in()?;
                    let body = self.parse_term()?;
                    Ok(Some(TermNode::Let(bind_var, value, body).into()))
                },
                Token::Hole(loc, name, contents) => {
                    self.consume();
                    Ok(Some(TermNode::Hole(HoleInfo::new(self.generate_hole_id(), name, contents, loc)).into()))
                },
                Token::Str(_loc, contents) => {
                    self.consume();
                    Ok(Some(TermNode::StrLit(contents).into()))
                },
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn generate_hole_id(&mut self) -> HoleId {
        let hole_id = self.next_hole_id;
        self.next_hole_id += 1;
        self.hole_count += 1;
        hole_id
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseErr> {
        Ok(self.consume_identifier_plus()?)
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseErr> {
        self.consume_expect_with()?;
        let idents = self.parse_pattern()?;
        self.consume_expect_fat_arrow()?;
        let body = self.parse_term()?;
        Ok(MatchArm(idents, body))
    }

    fn parse_match_arm_star(&mut self) -> Result<Vec<MatchArm>, ParseErr> {
        let mut match_arms = Vec::new();
        while let Some(Token::With(_)) = self.peek() {
            match_arms.push(self.parse_match_arm()?);
        }
        Ok(match_arms)
    }

    fn parse_match(&mut self) -> Result<Term, ParseErr> {
        self.consume_expect_match()?;
        let discriminee = self.parse_term()?;
        let match_arms = self.parse_match_arm_star()?;
        Ok(TermNode::Match(discriminee, match_arms).into())
    }

    fn parse_type_part(&mut self) -> Result<Type, ParseErr> {
        match self.peek() {
            Some(Token::LeftParen(_)) => {
                self.consume_expect_left_paren()?;
                let typ = self.parse_type()?;
                self.consume_expect_right_paren()?;
                return Ok(typ);
            },
            Some(Token::Ident(_, _name)) => {
                let ident = self.consume_identifier()?;
                Ok(TypeNode::Atom(ident).into())
            },
            None => Err("Expected '(' or identifier, but found end of input".to_string()),
            _ => Err(format!("Expected '(' or identifier, but found {:?}", self.peek().unwrap())),
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParseErr> {
        let mut type_parts = vec![self.parse_type_part()?];
        while let Some(Token::Arrow(_)) = self.peek() {
            self.consume_expect_arrow()?;
            type_parts.push(self.parse_type_part()?);
        }

        type_parts.reverse();

        let (first, rest) = type_parts.split_first().unwrap();
        let term: Type = rest.to_vec().iter().fold(first.clone(), |acc, cod| TypeNode::Arrow(cod.clone(), acc.clone()).into());
        Ok(term)
    }

    fn parse_term(&mut self) -> Result<Term, ParseErr> {
        if let Some(Token::Match(_)) = self.peek() {
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

            let mut term = if args.is_empty() {
                func
            } else {
                TermNode::App(func, args).into()
            };

            if let Some(Token::As(_)) = self.peek() {
                self.consume_expect_as()?;
                let typ = self.parse_type()?;
                term = TermNode::As(term, typ).into();
            }

            Ok(term)
        }
    }

    fn parse_def(&mut self) -> Result<Def, ParseErr> {
        self.consume_expect_def()?;
        let binding_name = self.consume_identifier()?;
        self.consume_expect_colon()?;
        let typ = self.parse_type()?;
        self.consume_expect_equals()?;
        let body = self.parse_term()?;
        Ok(Def(binding_name.to_string(), typ, body))
    }

    fn parse_import(&mut self) -> Result<Import, ParseErr> {
        self.consume_expect_import()?;
        let import_name = self.consume_identifier()?;
        Ok(Import(import_name))
    }

    fn parse_module(&mut self) -> Result<Module, ParseErr> {
        let mut definitions = Vec::new();
        let mut imports = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::Def(_) => {
                    let definition = self.parse_def()?;
                    definitions.push(definition );
                },
                Token::Import(_) => {
                    let import = self.parse_import()?;
                    imports.push(import);
                },
                _ => {
                    return Err(format!("Expected an item declaration, found {:?}", token));
                },
            }
        }
        Ok(Module::new(definitions, imports))
    }
}

pub fn parse_term(starting_hole_id: HoleId, source: Option<&Path>, input: &str) -> Result<(Term, u64), ParseErr> {
    let mut toker = Tokenizer::new(source, input);
    let tokens = toker.tokenize()?;

    let mut parser = Parser::new(starting_hole_id, tokens);

    let term = parser.parse_term()?;
    Ok((term, parser.number_of_holes()))
}

pub fn parse_module(starting_hole_id: HoleId, source: Option<&Path>, input: &str) -> Result<(Module, u64), ParseErr> {
    let mut toker = Tokenizer::new(source, input);
    let tokens = toker.tokenize()?;

    let mut parser = Parser::new(starting_hole_id, tokens);

    let module = parser.parse_module()?;
    Ok((module, parser.number_of_holes()))
}

pub fn parse_import(source: Option<&Path>, input: &str) -> Result<Import, ParseErr> {
    let mut toker = Tokenizer::new(source, input);
    let tokens = toker.tokenize()?;

    let mut parser = Parser::new(0 as HoleId, tokens);
    parser.parse_import()
}

pub fn parse_def(starting_hole_id: HoleId, source: Option<&Path>, input: &str) -> Result<Def, ParseErr> {
    let mut toker = Tokenizer::new(source, input);
    let tokens = toker.tokenize()?;

    let mut parser = Parser::new(starting_hole_id, tokens);
    parser.parse_def()
}
