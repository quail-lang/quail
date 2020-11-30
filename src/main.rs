pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;

fn main() {
//    let program = "(fun x => (fun y => y)) 1 2";
//    let program = "let x = 5 in succ x";
    let program = "(fun x y => y) 2 3";
    match parser::parse(program) {
        Ok(term) => println!("{:?}", eval::eval(term)),
        Err(e) => eprintln!("There was an error {:?}", e),
    }
}
