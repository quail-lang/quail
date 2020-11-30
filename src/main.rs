pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;

use std::fs;
use std::env;

use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut program = String::new();
    fs::File::open(filename).expect("File doesn't exist").read_to_string(&mut program).expect("Couldn't read from file");
    match parser::parse(program) {
        Ok(term) => println!("{:?}", eval::eval(term)),
        Err(e) => eprintln!("There was an error {:?}", e),
    }
}
