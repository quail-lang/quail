use std::collections::HashMap;
use std::rc;
use std::fmt;

use dirs;
use rustyline::error::ReadlineError;

use crate::parser;
use crate::ast;
use crate::hole;
use crate::builtins;
use crate::typecheck;

use ast::TermNode;
use ast::Def;
use ast::HoleId;
use ast::Import;
use ast::MatchArm;
use ast::Term;
use ast::Tag;
use builtins::TypeDef;
use crate::typecontext::TypeContext;

use std::path::{Path, PathBuf};

///
/// Runtime is the global store for all of the information loaded into the program.
///
#[derive(Debug)]
pub struct Runtime {
    /// Keeps track of which modules have been loaded into the Runtime
    /// already. This is currently being used to break cyclic imports.
    pub imports: Vec<String>,

    pub import_base: PathBuf,

    /// holes - keeps track of what values have been supplied for each hole.
    pub holes: HashMap<HoleId, Value>,

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

    /// This keeps track of the number of holes. This information is not
    /// captured by the holes field because the holes HashMap is sparse. The
    /// number_of_holes value is used to inform the parser which HoleID to start with
    /// as it parses a new term.
    pub number_of_holes: u64,

    /// This keeps track of the builtin typedef data for inductive
    /// types, such as Nat and Bool.
    pub inductive_typedefs: HashMap<String, TypeDef>,

    /// Tracks the value and types of all of the
    /// definitions of top-level bindings that have been loaded into the Runtime.
    pub definition_ctx: Context,
    /// Tracks types of all of the definitions of top-level bindings
    /// that have been loaded into the Runtime.
    pub definition_type_ctx: TypeContext,

    /// Tracks the values of the builtins, like println and show.
    pub builtin_ctx: Context,
    /// Tracks the types of the builtins, like println and show.
    pub builtin_type_ctx: TypeContext,
}

impl Runtime {
    /// Creates a new Runtime with no modules loaded. The inductive typedefs are defined, as well as the builtins.
    /// Sets up the history file, creating it if it doesn't exist.
    pub fn new() -> Self {
        let import_base_string = std::env::var("QUAIL_IMPORT_BASE").unwrap_or(".".to_string());
        let import_base = PathBuf::from(&import_base_string).canonicalize().unwrap();
        if !import_base.is_dir() {
            panic!("QUAIL_IMPORT_BASE is {} but that directory does not exist.", import_base_string);
        }

        let readline_file = dirs::config_dir()
            .expect("User does not have a home directory??")
            .join("quail").join("history");

        if !readline_file.exists() {
            std::fs::create_dir_all(&readline_file.parent().unwrap()).unwrap();
            std::fs::File::create(&readline_file).expect("Could not create readline file");
        }

        let inductive_typedefs: HashMap<String, TypeDef> = builtins::builtin_inductive_typedefs()
            .iter()
            .map(|itd| (itd.name.to_string(), itd.clone()))
            .collect();

        let mut builtin_ctx = builtins::builtins_ctx();
        let mut builtin_type_ctx = builtins::builtins_type_ctx();

        for inductive_typedef in inductive_typedefs.values() {
            builtin_ctx = builtin_ctx.append(inductive_typedef.ctor_context());
            builtin_type_ctx = builtin_type_ctx.append(inductive_typedef.ctor_type_context());
        }

        let mut editor = rustyline::Editor::new();
        if editor.load_history(&readline_file).is_err() {
            eprintln!("Could not read from {:?} for readline history.", &readline_file);
        }

        Runtime {
            imports: vec![],
            import_base,
            holes: HashMap::new(),
            readline_file: readline_file.to_string_lossy().to_string(),
            editor,
            number_of_holes: 0,

            inductive_typedefs,

            definition_ctx: Context::empty(),
            builtin_ctx,

            definition_type_ctx: TypeContext::empty(),
            builtin_type_ctx,
        }
    }

    /// Imports a module. `name` is the name of the module. The filepath is given by
    /// the name of the module with `.ql` appended to the end. Modules are searched in
    /// the user's current working directory.
    pub fn import(&mut self, name: &str) -> Result<(), RuntimeError> {
        let mut import_filename = name.to_string();
        import_filename.push_str(".ql");
        let import_filename = &Path::new(&import_filename);
        self.load_module(import_filename, &self.import_base.clone(), false)
    }

    pub fn load(&mut self, filepath: impl AsRef<Path>) -> Result<(), RuntimeError> {
        let basedir = Path::new(filepath.as_ref().parent().expect("Invalid path"));
        let filename = Path::new(filepath.as_ref().file_name().expect("Invalid path"));
        self.load_module(filename, basedir, true)
    }

    /// Loads a  a module. `filename` is given without the directory part. For instance,
    /// "main.ql". `basedir`, on the other hand, is the name of the directory. So the actual
    /// filepath would be `basedir.join(filename)`. We pass `basedir` in as a separate part so
    /// that we can import any modules specified by this module.
    ///
    /// `is_main` dictates whether this is the first module to be loaded by the ruletime, and
    /// thus, the one whose main we should use. When `load_module` calls itself to import a
    /// module recursively, it will do so with `is_main` set to `false`.
    ///
    /// Loading a module will have the effect of parsing the file, adding any new holes found
    /// in it, recursively loading any imports found in the file, typechecking any definitions,
    /// and then adding those definitions to the Runtime.
    fn load_module(
        &mut self,
        filename: &Path,
        basedir: &Path,
        is_main: bool,
    ) -> Result<(), RuntimeError> {
        if self.imports.contains(&filename.to_string_lossy().to_string()) {
            return Ok(());
        } else {
            self.imports.push(filename.to_string_lossy().to_string());
        }
        let filepath = basedir.join(filename);
        use std::fs;
        use std::io::Read;
        let mut module_text = String::new();
        fs::File::open(&filepath)?.read_to_string(&mut module_text)?;

        let (module, number_of_new_holes) = parser::parse_module(self.next_hole_id(), Some(filename), &module_text)?;
        self.add_holes(number_of_new_holes);

        for import in module.imports {
            let Import(name) = import;

            let mut import_filename = name.to_string();
            import_filename.push_str(".ql");
            let import_filename = &Path::new(&import_filename);

            self.load_module(import_filename, basedir, false)?;
        }

        for definition in module.definitions.iter() {
            let Def(name, typ, body) = definition;
            if is_main || name != "main" {
                let type_context = self.builtin_type_ctx.append(self.definition_type_ctx.clone()).extend(name, typ.clone());
                typecheck::check_type(&body, type_context, &self.inductive_typedefs, typ.clone())?;
                self.definition_type_ctx = self.definition_type_ctx.extend(&name.to_string(), typ.clone());

                let body_value = self.eval(&body, Context::empty());
                self.definition_ctx = self.definition_ctx.extend(&name.to_string(), body_value);
            }
        }
        Ok(())
    }

    /// Append a new definition to the Runtime after typechecking it.
    pub fn define(&mut self, definition: &Def) -> Result<(), RuntimeError> {
        let Def(name, typ, body) = definition;
        let type_context = self.builtin_type_ctx.append(self.definition_type_ctx.clone()).extend(&name, typ.clone());
        typecheck::check_type(&body, type_context, &self.inductive_typedefs, typ.clone())?;
        self.definition_type_ctx = self.definition_type_ctx.extend(&name.to_string(), typ.clone());

        let body_value = self.eval(&body, Context::empty());
        self.definition_ctx = self.definition_ctx.extend(&name.to_string(), body_value);
        Ok(())
    }

    /// Calculate the HoleId to the next hole loaded into the Runtime. Used as an input whenever
    /// running the parser.
    pub fn next_hole_id(&self) -> HoleId {
        self.number_of_holes as HoleId
    }

    pub fn add_holes(&mut self, number_of_holes: u64) {
        self.number_of_holes += number_of_holes;
    }

    pub fn exec(&mut self) {
        self.definition_ctx.lookup("main", 0).expect("There should be a main in your module");
    }

    /// Reads a line using the rustyline readline library and saves it to the user's history file.
    pub fn readline(&mut self) -> Result<String, ReadlineError> {
        let line = self.editor.readline("> ")?;
        self.editor.add_history_entry(line.as_str());
        self.editor.save_history(&self.readline_file)?;
        Ok(line)
    }

    /// Fills a given hole with a given value.
    pub fn fill_hole(&mut self, hole_id: HoleId, value: Value) {
        self.holes.insert(hole_id, value);
    }

    /// Retrieves a value for the given hole.
    pub fn hole_value(&self, hole_id: HoleId) -> Option<Value> {
        assert!((hole_id as u64) < self.number_of_holes, "Invalid HoleId!");
        self.holes.get(&hole_id).cloned()
    }

    /// Applies to a function its list of arguments and returns the result.
    fn apply(self: &mut Runtime, func: Value, args: Vec<Value>) -> Value {
        match &func {
            Value::Fun(x, body, local_ctx) => {
                match args.clone().split_first() {
                    None => func,
                    Some((v, vs_remaining)) => {
                        let new_ctx = local_ctx.extend(&x, v.clone());
                        let new_func = self.eval(&body, new_ctx);
                        let new_func = self.force(&new_func);
                        self.apply(new_func, vs_remaining.to_vec())
                    },
                }
            },
            Value::Ctor(tag, contents) => {
                let mut new_contents = contents.clone();
                new_contents.extend(args);
                Value::Ctor(tag.to_string(), new_contents)
            },
            Value::CoCtor(tag, contents) => {
                let mut new_contents = contents.clone();
                new_contents.extend(args);
                Value::CoCtor(tag.to_string(), new_contents)
            },
            Value::Prim(prim) => {
                let args = args.into_iter().map(|a| self.force_deep(&a)).collect();
                prim(self, args)
            },
            _ => panic!(format!("Applied arguments to non-function {:?}", func)),
        }
    }

    /// Evaluates a term in a given local context and returns the result.
    pub fn eval(self: &mut Runtime, t: &TermNode, ctx: Context) -> Value {
        match t {
            TermNode::Var(x, k) => {
                ctx.lookup(&x, *k)
                    .or_else(|| self.definition_ctx.lookup(&x, *k))
                    .or_else(|| self.builtin_ctx.lookup(&x, *k))
                    .expect(&format!("Unbound variable {:?}", &x))
            },
            TermNode::StrLit(contents) => Value::Str(contents.to_string()),
            TermNode::Lam(x, body) => {
                Value::Fun(x.clone(), body.clone(), ctx.clone())
            },
            TermNode::App(f, vs) => {
                let f_value = self.eval(&f, ctx.clone());
                let f_value = self.force(&f_value);
                let vs_values: Vec<Value> = vs.iter()
                    .map(|v| Value::Thunk(v.clone(), ctx.clone()))
                    .collect();
                self.apply(f_value, vs_values)
            },
            TermNode::Let(x, v, body) => {
                let v_value = self.eval(&v, ctx.clone());
                let extended_ctx = ctx.extend(x, v_value);
                self.eval(&body, extended_ctx)
            },
            TermNode::Match(t, match_arms) => {
                let t_value = self.eval(&t, ctx.clone());
                let t_value = self.force(&t_value);
                match t_value {
                    Value::Ctor(tag, contents) => {
                        let MatchArm(pat, body) = ast::find_matching_arm(&tag, &match_arms);

                        let bind_names: Vec<String> = pat[1..].to_vec();
                        let bind_values: Vec<Value> = contents.clone();
                        let bindings: Vec<(String, Value)> = bind_names.into_iter().zip(bind_values).collect();

                        let extended_ctx = ctx.extend_many(&bindings);
                        self.eval(&body, extended_ctx)
                    },
                    Value::CoCtor(tag, contents) => {
                        let MatchArm(pat, body) = ast::find_matching_arm(&tag, &match_arms);

                        let bind_names: Vec<String> = pat[1..].to_vec();
                        let bind_values: Vec<Value> = contents.clone();
                        let bindings: Vec<(String, Value)> = bind_names.into_iter().zip(bind_values).collect();

                        let extended_ctx = ctx.extend_many(&bindings);
                        self.eval(&body, extended_ctx)
                    },
                    _ => panic!(format!("Expected a constructor during match statement, but found {:?}", &t_value)),
                }
            },
            TermNode::Hole(hole_info) => hole::fill(self, hole_info, ctx),
            TermNode::As(term, _typ) => self.eval(&term, ctx),
        }
    }

    pub fn force(&mut self, value: &Value) -> Value {
        let mut result = value.clone();
        while let Value::Thunk(t, ctx) = &result {
            result = self.eval(&t, ctx.clone());
        }
        result
    }

    pub fn force_deep(&mut self, value: &Value) -> Value {
        let mut result = value.clone();
        while let Value::Thunk(t, ctx) = result {
            result = self.eval(&t, ctx.clone());
        }

        if let Value::Ctor(tag, contents) = result {
            let contents = contents.iter().map(|v| self.force_deep(v)).collect();
            result = Value::Ctor(tag.to_string(), contents);
        }
        result
    }
}

#[derive(Debug)]
pub struct RuntimeError(String);

impl std::convert::From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        RuntimeError(error.to_string())
    }
}

impl std::convert::From<String> for RuntimeError {
    fn from(error: String) -> Self {
        RuntimeError(error)
    }
}

#[derive(Clone)]
pub enum Value {
    Ctor(Tag, Vec<Value>),
    CoCtor(Tag, Vec<Value>),
    Fun(String, Term, Context),
    Prim(rc::Rc<dyn Fn(&mut Runtime, Vec<Value>) -> Value>),
    Str(String),
    Thunk(Term, Context),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Ctor(tag, contents) => {
                write!(f, "{}", &tag)?;
                for value in contents {
                    write!(f, " ({:?})", value)?;
                }
                Ok(())
            },
            Value::CoCtor(tag, contents) => {
                write!(f, "{}", &tag)?;
                for value in contents {
                    write!(f, " ({:?})", value)?;
                }
                Ok(())
            },
            Value::Str(s) => write!(f, "{:?}", s),
            Value::Fun(_, _, _) => write!(f, "<fun>"),
            Value::Prim(_) => write!(f, "<prim>"),
            Value::Thunk(_, _) => write!(f, "<thunk>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context(rc::Rc<ContextNode>);

impl Context {
    pub fn empty() -> Self {
        Context(rc::Rc::new(ContextNode(Vec::new())))
    }

    pub fn lookup(&self, x: &str, k: usize) -> Option<Value> {
        let Context(rc_ctx_node) = self;
        let ContextNode(var_val_list) = rc_ctx_node.as_ref();
        for (y, value) in var_val_list.iter().rev() {
            if x == y {
                if k == 0 {
                    return Some(value.clone());
                } else {
                    return self.lookup(x, k - 1);
                }
            }
        }
        None
    }

    pub fn extend(&self, x: &str, v: Value) -> Context {
        let Context(rc_ctx_node) = self;
        let ContextNode(var_val_list) = rc_ctx_node.as_ref();
        let mut extended_var_val_list = var_val_list.clone();
        extended_var_val_list.push((x.to_string(), v.clone()));
        Context(rc::Rc::new(ContextNode(extended_var_val_list)))
    }

    pub fn extend_many(&self, bindings: &[(String, Value)]) -> Context {
        let mut ctx = self.clone();
        for (name, value) in bindings.iter() {
            ctx = ctx.extend(name, value.clone());
        }
        ctx
    }

    pub fn append(&self, ctx: Context) -> Context {
        let mut result_ctx = self.clone();
        for (name, value) in ctx.bindings().iter() {
            result_ctx = result_ctx.extend(name, value.clone());
        }
        result_ctx
    }

    pub fn bindings(&self) -> Vec<(String, Value)> {
        let Context(context_node_rc) = self;
        let ContextNode(bindings) = context_node_rc.as_ref();
        bindings.clone()
    }
}

#[derive(Debug)]
struct ContextNode(Vec<(String, Value)>);
