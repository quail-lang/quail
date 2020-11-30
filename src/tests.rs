#![cfg(test)]
use std::fs;

use crate::runtime::Runtime;

#[test]
fn run_examples() {
    let paths = fs::read_dir("examples").expect("Could not open examples/ directory");
    for path in paths {
        let filename = path.expect("Couldn't open file").path();
        println!("{:?}", filename);
        let mut runtime = Runtime::new();
        runtime.load(filename).unwrap();
        runtime.exec();
    }
}
