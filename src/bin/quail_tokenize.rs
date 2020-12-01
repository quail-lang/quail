use quail::runtime;
use quail::runtime::ImportResolver;
use quail::tokenizer::*;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Quail Tokenizer", about = "Tokenizer for The Quail Programming Language")]
struct Opt {
        #[structopt(help = "Input file")]
        filename: String,
}

fn main() -> Result<(), runtime::RuntimeError> {
    let opt = Opt::from_args();

    let mut import_resolver = runtime::ChainedImportResolver::new(
        Box::new(runtime::FilePathImportResolver),
        Box::new(runtime::FileImportResolver::new("examples")),
    );

    let mut module_text = String::new();
    import_resolver.resolve(&opt.filename)?.reader.read_to_string(&mut module_text)?;

    let tokss = tokenize_lines(None, &module_text)?;
    let module_text_lines: Vec<String> = module_text.lines().map(|l| l.to_owned()).collect();
    assert_eq!(tokss.len(), module_text_lines.len());

    for (toks, line) in tokss.iter().zip(module_text_lines.iter()) {
        println!("    {}", line);
        print!("#   ");
        for tok in toks {
            print!("{} ", tok.show());
        }
        println!();
        println!();
    }

    Ok(())
}
