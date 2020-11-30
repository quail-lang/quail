use rustyline::error::ReadlineError;

use crate::ast;
use crate::runtime;
use crate::parser;
use crate::typecheck;

use ast::Context;
use runtime::Runtime;

pub fn repl(runtime: &mut Runtime) {
    loop {
        match runtime.readline() {
            Ok(term_text) => {
                match parser::parse_term(runtime.next_hole_id(), None, &term_text) {
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
            },
            Err(ReadlineError::Interrupted) => (),
            Err(ReadlineError::Eof) => std::process::exit(1),
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
