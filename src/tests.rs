#![cfg(test)]
use std::fs;

use crate::eval;

#[test]
fn run_examples() {
    let paths = fs::read_dir("examples").expect("Could not open examples/ directory");
    for path in paths {
        let filename = path.expect("Couldn't open file").path();
        println!("{:?}", filename);
        let mut runtime = eval::Runtime::load(filename);
        runtime.exec();
    }
}
