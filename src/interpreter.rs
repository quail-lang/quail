use rustyline::error::ReadlineError;

use crate::ast;
use crate::runtime;
use crate::parser;
use crate::typecheck;

use ast::Context;
use ast::Import;
use ast::Def;
use runtime::Runtime;

pub fn repl(runtime: &mut Runtime) {
    loop {
        match runtime.readline() {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    ()
                } else if line.starts_with("import") {
                    match parser::parse_import(None, &line) {
                        Ok(Import(module_name)) => {
                            match runtime.import(&module_name) {
                                Ok(()) => println!("import successful"),
                                Err(msg) => println!("{:?}", msg),
                            }
                        },
                        Err(e) => println!("There was an error {:?}", e),
                    }
                } else if line.starts_with("def") {
                    match parser::parse_def(runtime.next_hole_id(), None, &line) {
                        Ok(definition) => {
                            match runtime.define(&definition) {
                                Ok(()) => {
                                    let Def(name, typ, _body) = &definition;
                                    println!("=> {} : {}", name, typ);
                                },
                                Err(err) => println!("Error: {:?}", err),
                            }
                        },
                        Err(e) => println!("There was an error {:?}", e),
                    }
                } else {
                    match parser::parse_term(runtime.next_hole_id(), None, &line) {
                        Ok((term, number_of_new_holes)) => {
                            let type_context = runtime.builtin_type_ctx.append(runtime.definition_type_ctx.clone());
                            match typecheck::infer_type(
                                    term.clone(),
                                    type_context,
                                    &runtime.inductive_typedefs,
                                ) {
                                Ok(typ) => {
                                    runtime.add_holes(number_of_new_holes);
                                    let value = runtime.eval(term, Context::empty());
                                    println!("=> {:?} : {}", &value, &typ);
                                },
                                Err(type_error) => println!("Type Error: {:?}", &type_error),
                            }
                        },
                        Err(e) => println!("There was an error {:?}", e),
                    }
                }
            },
            Err(ReadlineError::Interrupted) => (),
            Err(ReadlineError::Eof) => std::process::exit(1),
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
