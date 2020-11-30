pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;
pub mod tests;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut runtime = eval::Runtime::load(filename);
    runtime.exec();
}
