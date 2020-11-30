pub mod tokenizer;
pub mod parser;
pub mod ast;
pub mod runtime;
pub mod typecheck;
pub mod hole;
pub mod builtins;

pub mod tests;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "quail", about = "The Quail Programming Language")]
struct Opt {
        #[structopt(help = "Input file")]
        filename: Option<String>,
}

fn main() {
    let opt = Opt::from_args();
    let filename = opt.filename;
    match filename {
        None => {
            println!("{}", include_str!("../assets/quail.txt"));
        },
        Some(filename) => {
            let mut runtime = runtime::Runtime::new();
            runtime.load(filename);
            runtime.exec();
        },
    }
}
