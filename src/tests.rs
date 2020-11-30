#![cfg(test)]
use std::fs;

use crate::runtime::Runtime;
use crate::runtime::FileImportResolver;

#[test]
fn run_examples() {
    let paths = fs::read_dir("examples").expect("Could not open examples/ directory");
    for path in paths {
        let filename = path.expect("Couldn't open file").path();
        println!("{:?}", filename);
        let mut runtime = Runtime::new();
        let mut import_resolver = FileImportResolver::new("examples");
        let import_name = filename.file_stem().unwrap().to_str().unwrap();
        runtime.import(&import_name, &mut import_resolver, true).unwrap();
        runtime.exec();
    }
}
