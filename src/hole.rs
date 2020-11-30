use crate::eval::Runtime;
use crate::ast;
use crate::ast::Context;
use crate::eval;
use crate::ast::Value;
use crate::ast::HoleInfo;
use crate::parser;

use rustyline::error::ReadlineError;

pub fn fill(runtime: &mut Runtime, hole_info: &HoleInfo, ctx: Context) -> Value {
    match runtime.holes.get_mut(&hole_info.hole_id) {
        Some(value) => value.clone(),
        None => {
            introduce_hole(hole_info);
            show_bindings(&ctx);
            show_globals(runtime);

            loop {
                match runtime.readline() {
                    Ok(term_text) => {
                        match parser::parse_term(term_text) {
                            Ok(term) => {
                                return eval::eval(term, ctx, runtime);
                            }
                            Err(e) => println!("There was an error {:?}", e),
                        }
                    },
                    Err(ReadlineError::Interrupted) => (),
                    Err(ReadlineError::Eof) => std::process::exit(1),
                    Err(err) => {
                        panic!("Error: {:?}", err);
                    }
                }
            }
        }
    }
}

fn introduce_hole(hole_info: &HoleInfo) {
    match &hole_info.name {
        None => {
            println!("Encountered hole: #{}", hole_info.hole_id);
            println!("");
        }
        Some(name) => {
            println!("Encountered hole: {}", name);
            println!("");
        }
    }

    if let Some(contents_string) = &hole_info.contents {
        println!("    Note: {:?}", contents_string);
        println!("");
    }

}

fn show_bindings(ctx: &Context) {
    println!("    Bindings:");
    for (name, value) in ctx.bindings().into_iter() {
        println!("        {} = {:?}", name, &value);
    }
    println!("");
}

fn show_globals(runtime: &Runtime) {
    println!("    Globals:");
    for definition in runtime.definitions.iter() {
        let ast::Def(name, _) = definition;
        println!("        {}", &name);
    }
    println!("");
}
