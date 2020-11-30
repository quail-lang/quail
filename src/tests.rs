#![cfg(test)]
use std::fs;
use std::io;

use crate::parser;
use crate::eval;

use std::io::Read;

#[test]
fn a_test() -> io::Result<()> {
    let paths = fs::read_dir("examples")?;
    let filename = "";
    for path in paths {
        let filename = path?.path();
        println!("{:?}", filename);
        let mut program_text = String::new();
        fs::File::open(filename).expect("File doesn't exist").read_to_string(&mut program_text).expect("Couldn't read from file");

        match parser::parse_program(program_text) {
            Ok(program) => eval::exec(&program),
            Err(e) => eprintln!("There was an error {:?}", e),
        }
    }
    Ok(())
}
