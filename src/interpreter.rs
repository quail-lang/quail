use rustyline::error::ReadlineError;

use crate::ast;
use crate::runtime;
use crate::parser;
use crate::typecheck;

use ast::Import;
use ast::Def;
use runtime::Runtime;
use runtime::Context;

pub fn repl(runtime: &mut Runtime) {
    loop {
        match runtime.readline() {
            Ok(line) => {
                repl_line(runtime, &line);
            },
            Err(ReadlineError::Interrupted) => (),
            Err(ReadlineError::Eof) => std::process::exit(1),
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}

fn repl_line(runtime: &mut Runtime, line: &str) {
    let line = line.trim();

    if line.is_empty() {
        ()
    } else if line.starts_with("import") {
        repl_line_import(runtime, line);
    } else if line.starts_with("def") {
        repl_line_def(runtime, line);
    } else {
        repl_line_term(runtime, line);
    }
}

fn repl_line_import(runtime: &mut Runtime, line: &str) {
    match parser::parse_import(None, line) {
        Ok(Import(module_name)) => {
            match runtime.import(&module_name) {
                Ok(()) => println!("import successful"),
                Err(msg) => println!("{:?}", msg),
            }
        },
        Err(e) => println!("There was an error {:?}", e),
    }
}

fn repl_line_def(runtime: &mut Runtime, line: &str) {
    match parser::parse_def(runtime.next_hole_id(), None, &line) {
        Ok(definition) => {
            match runtime.define(&definition) {
                Ok(()) => {
                    let Def(name, typ, _body) = definition;
                    println!("=> {} : {}", name, *typ);
                },
                Err(err) => println!("Error: {:?}", err),
            }
        },
        Err(e) => println!("There was an error {:?}", e),
    }
}

fn repl_line_term(runtime: &mut Runtime, line: &str) {
    match parser::parse_term(runtime.next_hole_id(), None, &line) {
        Ok((term, number_of_new_holes)) => {
            let type_context = runtime.builtin_type_ctx.append(runtime.definition_type_ctx.clone());
            match typecheck::infer_type(
                    &term,
                    type_context,
                    &runtime.inductive_typedefs,
                ) {
                Ok(typ) => {
                    runtime.add_holes(number_of_new_holes);
                    let value = runtime.eval(&term, Context::empty());
                    println!("=> {:?} : {}", &value, *typ);
                },
                Err(type_error) => println!("Type Error: {:?}", &type_error),
            }
        },
        Err(e) => println!("There was an error {:?}", e),
    }
}
