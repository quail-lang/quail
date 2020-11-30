pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;

fn main() {
    println!("{:?}", parser::parse("fn x => (f x)"));
}
