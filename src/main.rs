pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;

fn main() {
//    let program = "(fun x => (fun y => y)) 1 2";
    let program = "(fun f => f (f (f 1))) succ";
    match parser::parse(program) {
        Ok(term) => println!("{:?}", eval::eval(term)),
        Err(e) => eprintln!("There was an error {:?}", e),
    }
}
