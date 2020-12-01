use quail::runtime;
use quail::interpreter;
use quail::resolver;

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
            println!("{}", include_str!("../../assets/quail.txt"));
            let mut interpreter = interpreter::Interpreter::new();
            interpreter::repl(&mut interpreter);
        },
        Some(filename) => {
            let mut runtime = runtime::Runtime::new();
            let mut import_resolver = resolver::ChainedImportResolver::new(
                Box::new(resolver::FilePathImportResolver),
                Box::new(resolver::FileImportResolver::new("examples")),
            );
            runtime.import(&filename, &mut import_resolver, true)?;
            runtime.exec();
        },
    }
    Ok(())
}
