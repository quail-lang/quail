use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;
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

    fn parse_term_part(&mut self) -> Result<Option<Term>, ParseErr> {
        match self.peek() {
            Some(token) => match token {
                Token::Ident(_, name) => {
                    self.consume();
                    Ok(Some(TermNode::Var(name).into()))
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

    fn parse_match_arm_plus(&mut self) -> Result<Vec<MatchArm>, ParseErr> {
        let mut match_arms = Vec::new();
        match_arms.push(self.parse_match_arm()?);
        while let Some(Token::With(_)) = self.peek() {
            match_arms.push(self.parse_match_arm()?);
        }
        Ok(match_arms)
    }

    fn parse_match(&mut self) -> Result<Term, ParseErr> {
        self.consume_expect_match()?;
        let discriminee = self.parse_term()?;
        let match_arms = self.parse_match_arm_plus()?;
        Ok(TermNode::Match(discriminee, match_arms).into())
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

            if args.is_empty() {
                Ok(func)
            } else {
                Ok(TermNode::App(func, args).into())
            }
        }
    }

    fn parse_def(&mut self) -> Result<Def, ParseErr> {
        self.consume_expect_def()?;
        let idents = self.consume_identifier_plus()?;
        let (binding_name, var_names) = idents.split_first().unwrap();
        self.consume_expect_equals()?;
        let mut body = self.parse_term()?;
        for var_name in var_names.iter().rev() {
            body = TermNode::Lam(var_name.to_string(), body).into();
        }
        Ok(Def(binding_name.to_string(), body))
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

pub fn parse_term(input: &str) -> Result<Term, ParseErr> {
    let mut toker = Tokenizer::new(input);
    let tokens = toker.tokenize().expect("Error when tokenizing");

    let mut parser = Parser::new(tokens);

    parser.parse_term()
}

pub fn parse_module(input: &str) -> Result<Module, ParseErr> {
    let mut toker = Tokenizer::new(input);
    let tokens = toker.tokenize().expect("Error when tokenizing");

    let mut parser = Parser::new(tokens);

    parser.parse_module()
}
