pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;
pub mod tests;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("{}", include_str!("../assets/quail.txt"));
    let mut runtime = eval::Runtime::load(filename);
    runtime.exec();
}
