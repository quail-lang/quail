pub mod tokenizer;
pub mod parser;
pub mod ast;
pub mod eval;
pub mod typecheck;
pub mod hole;
pub mod builtins;

pub mod tests;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "quail", about = "The Quail Programming Language")]
struct Opt {
        #[structopt(help = "Input file")]
        filename: String,
}

fn main() {
    let opt = Opt::from_args();
    let filename = opt.filename;
    println!("{}", include_str!("../assets/quail.txt"));
    let mut runtime = eval::Runtime::load(filename);
    runtime.exec();
}
