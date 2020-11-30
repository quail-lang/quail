use rustyline::error::ReadlineError;

use crate::ast;
use crate::runtime;
use crate::parser;

use ast::Value;
use ast::HoleInfo;
use ast::Context;
use ast::Def;
use runtime::Runtime;

pub fn fill(runtime: &mut Runtime, hole_info: &HoleInfo, ctx: Context) -> Value {
    match runtime.holes.get_mut(&hole_info.hole_id) {
        Some(value) => value.clone(),
        None => {
            introduce_hole(hole_info);
            show_bindings(&ctx);
            show_globals(runtime);

            loop {
                match runtime.readline() {
                    Ok(line) => {
                        match parse_command(&line) {
                            None => (),
                            Some(Command::Fill(term_text)) => {
                                match parser::parse_term(&term_text) {
                                    Ok(term) => {
                                        let value = runtime.eval(term, ctx.clone());
                                        println!("=> {:?}", &value);
                                        runtime.fill_hole(hole_info.hole_id, value.clone());
                                        return value;
                                    },
                                    Err(e) => println!("There was an error {:?}", e),
                                }
                            },
                            Some(Command::Eval(term_text)) => {
                                match parser::parse_term(&term_text) {
                                    Ok(term) => {
                                        let value = runtime.eval(term, ctx.clone());
                                        println!("=> {:?}", &value);
                                    },
                                    Err(e) => println!("There was an error {:?}", e),
                                }
                            },
                            Some(Command::Invalid(invalid_cmd)) => {
                                println!("Invalid command: {}", invalid_cmd);
                                println!("Hint: Try 'help' if you don't know what to do.");
                            },
                            Some(Command::Abort) => std::process::exit(1),
                            Some(Command::Help) => println!("{}", include_str!("../assets/help/hole.txt")),
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

enum Command {
    Fill(String),
    Eval(String),
    Abort,
    Help,
    Invalid(String),
}

fn introduce_hole(hole_info: &HoleInfo) {
    match &hole_info.name {
        None => {
            println!("Encountered hole: #{}", hole_info.hole_id);
            println!();
        }
        Some(name) => {
            println!("Encountered hole: {}", name);
            println!();
        }
    }

    if let Some(contents_string) = &hole_info.contents {
        println!("    Note: {:?}", contents_string);
        println!();
    }

}

fn show_bindings(ctx: &Context) {
    println!("    Bindings:");
    for (name, value) in ctx.bindings().into_iter() {
        println!("        {} = {:?}", name, &value);
    }
    println!();
}

fn show_globals(runtime: &Runtime) {
    println!("    Globals:");
    for definition in runtime.definitions.iter() {
        let Def(name, _) = definition;
        println!("        {}", &name);
    }
    println!();
}

fn parse_command(line: &str) -> Option<Command> {
    let parts: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();
    if parts.is_empty() {
        None
    } else {
        let command_name = &parts[0];
        if command_name == "fill" {
            let remainder: String = parts[1..].join(" ");
            Some(Command::Fill(remainder))
        } else if command_name =="eval" {
            let remainder: String = parts[1..].join(" ");
            Some(Command::Eval(remainder))
        } else if command_name =="abort" || command_name == "exit" || command_name == "quit" {
            Some(Command::Abort)
        } else if command_name == "help" || command_name == "h" || command_name == "?" {
            Some(Command::Help)
        } else {
            Some(Command::Invalid(command_name.to_string()))
        }

    }
}
