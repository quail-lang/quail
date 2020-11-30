pub mod tokenizer;
pub mod parser;
pub mod ast;
pub mod runtime;
pub mod typecheck;
pub mod hole;
pub mod builtins;
pub mod interpreter;
pub mod types;
pub mod typecontext;

pub mod tests;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Quail", about = "The Quail Programming Language")]
struct Opt {
        #[structopt(help = "Input file")]
        filename: Option<String>,
}

fn main() -> Result<(), runtime::RuntimeError> {
    let opt = Opt::from_args();
    let filename = opt.filename;
    match filename {
        None => {
            println!("{}", include_str!("../assets/quail.txt"));
            let mut runtime = runtime::Runtime::new();
            interpreter::repl(&mut runtime);
        },
        Some(filename) => {
            let mut runtime = runtime::Runtime::new();
            runtime.load(filename)?;
            runtime.exec();
        },
    }
    Ok(())
}
