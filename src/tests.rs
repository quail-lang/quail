#![cfg(test)]
use std::fs;

use crate::parser;
use crate::eval;

use std::io::Read;

#[test]
fn run_examples() {
    let paths = fs::read_dir("examples").expect("Could not open examples/ directory");
    for path in paths {
        let filename = path.expect("Couldn't open file").path();
        println!("{:?}", filename);
        let mut program_text = String::new();
        fs::File::open(filename).expect("File doesn't exist").read_to_string(&mut program_text).expect("Couldn't read from file");

        match parser::parse_program(program_text) {
            Ok(program) => eval::exec(&program),
            Err(e) => panic!("There was an error {:?}", e),
        }
    }
}
