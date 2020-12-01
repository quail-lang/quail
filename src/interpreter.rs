use dirs;
use rustyline::error::ReadlineError;

use crate::ast;
use crate::runtime;
use crate::parser;
use crate::typecheck;
use crate::resolver;

use ast::Import;
use ast::Def;
use runtime::Runtime;
use runtime::Context;

pub struct Interpreter {
    /// The REPL and hole-filling mode both use rustyline, which is
    /// a binding around readline. This filename needs to be preserved so that given
    /// a new line of input, it can record it to this file.
    ///
    /// This value uses the dirs::config_dir() function of the dirs crate. It will
    /// likely pick out something like $HOME/.config/quail/history as the location
    /// to save the user's history.
    pub readline_file: String,

    /// The rustyline Editor. This is a handle to interact with the readline library.
    pub editor: rustyline::Editor<()>,

    pub runtime: Runtime,
}

impl Interpreter {
    pub fn new() -> Self {
        let readline_file = dirs::config_dir()
            .expect("User does not have a home directory??")
            .join("quail").join("history");

        if !readline_file.exists() {
            std::fs::create_dir_all(&readline_file.parent().unwrap()).unwrap();
            std::fs::File::create(&readline_file).expect("Could not create readline file");
        }

        let mut editor = rustyline::Editor::new();
        if editor.load_history(&readline_file).is_err() {
            eprintln!("Could not read from {:?} for readline history.", &readline_file);
        }

        let runtime = Runtime::new();

        Interpreter {
            readline_file: readline_file.to_string_lossy().to_string(),
            editor,
            runtime,
        }
    }

    /// Reads a line using the rustyline readline library and saves it to the user's history file.
    pub fn readline(&mut self) -> Result<String, ReadlineError> {
        let line = self.editor.readline("> ")?;
        self.editor.add_history_entry(line.as_str());
        self.editor.save_history(&self.readline_file)?;
        Ok(line)
    }

}

pub fn repl(interpreter: &mut Interpreter) {
    loop {
        match interpreter.readline() {
            Ok(line) => {
                repl_line(&mut interpreter.runtime, &line);
            },
            Err(ReadlineError::Interrupted) => (),
            Err(ReadlineError::Eof) => std::process::exit(1),
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}

fn repl_line(runtime: &mut Runtime, line: &str) {
    let line = line.trim();

    if line.is_empty() {
        ()
    } else if line.starts_with("import") {
        repl_line_import(runtime, line);
    } else if line.starts_with("def") {
        repl_line_def(runtime, line);
    } else {
        repl_line_term(runtime, line);
    }
}

fn repl_line_import(runtime: &mut Runtime, line: &str) {
    let mut import_resolver = resolver::FileImportResolver::new("examples");

    match parser::parse_import(None, line) {
        Ok(Import(module_name)) => {
            match runtime.import(&module_name, &mut import_resolver, false) {
                Ok(()) => println!("import successful"),
                Err(msg) => println!("{:?}", msg),
            }
        },
        Err(e) => println!("There was an error {:?}", e),
    }
}

fn repl_line_def(runtime: &mut Runtime, line: &str) {
    match parser::parse_def(None, &line) {
        Ok(definition) => {
            match runtime.define(&definition) {
                Ok(()) => {
                    let Def(name, typ, _body) = definition;
                    println!("=> {} : {}", name, *typ);
                },
                Err(err) => println!("Error: {:?}", err),
            }
        },
        Err(e) => println!("There was an error {:?}", e),
    }
}

fn repl_line_term(runtime: &mut Runtime, line: &str) {
    match parser::parse_term(None, &line) {
        Ok(term) => {
            let type_context = runtime.builtin_type_ctx.append(runtime.definition_type_ctx.clone());
            match typecheck::infer_type(
                    &term,
                    type_context,
                    &runtime.inductive_typedefs,
                ) {
                Ok(typ) => {
                    let value = runtime.eval(&term, Context::empty());
                    println!("=> {:?} : {}", &value, *typ);
                },
                Err(type_error) => println!("Type Error: {:?}", &type_error),
            }
        },
        Err(e) => println!("There was an error {:?}", e),
    }
}
