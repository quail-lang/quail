use rustyline::error::ReadlineError;

use crate::ast;
use crate::runtime;
use crate::parser;

use ast::HoleInfo;
use ast::HoleId;
use runtime::Context;
use runtime::Runtime;
use runtime::Value;

pub fn fill(runtime: &mut Runtime, hole_info: &HoleInfo, ctx: Context) -> Value {
    match runtime.holes.get_mut(&hole_info.hole_id) {
        Some(value) => value.clone(),
        None => {
            introduce_hole(hole_info);
            show_bindings(&ctx);
            show_globals(runtime);
            show_holes(runtime, hole_info.hole_id);

            let mut confuse_count = 0;

            loop {
                match runtime.readline() {
                    Ok(line) => {
                        confuse_count = 0;
                        match parse_command(&line) {
                            None => (),
                            Some(command) => match exec_command(runtime, &command, hole_info, &ctx) {
                                None => (),
                                Some(value) => return value,
                            },
                        }
                    },
                    Err(ReadlineError::Interrupted) => {
                        confuse_count += 1;
                        if confuse_count > 1 {
                            println!("Use Ctrl-D to exit.");
                        }
                    },
                    Err(ReadlineError::Eof) => std::process::exit(1),
                    Err(err) => {
                        panic!("Error: {:?}", err);
                    }
                }
            }
        }
    }
}

fn exec_command(
    runtime: &mut Runtime,
    command: &Command,
    hole_info: &HoleInfo,
    ctx: &Context,
) -> Option<Value> {
    match command {
        Command::Fill(term_text) => {
            match parser::parse_term(runtime.next_hole_id(), None, &term_text) {
                Ok((term, number_of_new_holes)) => {
                    runtime.add_holes(number_of_new_holes);
                    let value = runtime.eval(&term, ctx.clone());
                    println!("=> {:?}", &value);
                    runtime.fill_hole(hole_info.hole_id, value.clone());
                    return Some(value);
                },
                Err(e) => println!("There was an error {:?}", e),
            }
        },
        Command::Eval(term_text) => {
            match parser::parse_term(runtime.next_hole_id(), None, &term_text) {
                Ok((term, number_of_new_holes)) => {
                    runtime.add_holes(number_of_new_holes);
                    let value = runtime.eval(&term, ctx.clone());
                    println!("=> {:?}", &value);
                },
                Err(e) => println!("There was an error {:?}", e),
            }
        },
        Command::Invalid(invalid_cmd) => {
            println!("Invalid command: {}", invalid_cmd);
            println!("Hint: Try 'help' if you don't know what to do.");
        },
        Command::Abort => std::process::exit(1),
        Command::Help => println!("{}", include_str!("../assets/help/hole.txt")),
    }

    None
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
            println!("Encountered hole: #{}     [{}]", hole_info.hole_id, hole_info.loc);
            println!();
        }
        Some(name) => {
            println!("Encountered hole: {}      [{}]", name, hole_info.loc);
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
    for (name, _value) in runtime.definition_ctx.bindings() {
        println!("        {}", &name);
    }
    println!();
}

fn show_holes(runtime: &Runtime, active_hole_id: HoleId) {
    println!("    Holes:");
    for hole_id in 0..runtime.number_of_holes {
        let hole_id = hole_id as HoleId;
        let leader = if hole_id == active_hole_id {
            "  > "
        } else {
            "    "
        };

        match runtime.hole_value(hole_id) {
            Some(value) => println!("    {}#{} = {:?}", leader, hole_id, value),
            None => println!("    {}#{}", leader, hole_id),
        }
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
