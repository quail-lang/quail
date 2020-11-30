pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;
pub mod tests;

use std::fs;
use std::env;

use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut program_text = String::new();
    fs::File::open(filename).expect("File doesn't exist").read_to_string(&mut program_text).expect("Couldn't read from file");

    match parser::parse_module(program_text) {
        Ok(module) => eval::exec(&module),
        Err(e) => eprintln!("There was an error {:?}", e),
    }
}
