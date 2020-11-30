pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;

fn main() {
    println!("{}", parser::parse_sexpr("(3 (a b c) 5)").unwrap());
    println!("{}", parser::parse_sexpr("(I (a b c) () (f x))").unwrap());
    println!("{:?}", parser::parse("(fn x (f x))"));
//    println!("{}", parser::parse("(f x)"));
}
