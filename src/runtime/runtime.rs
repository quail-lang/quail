use std::collections::HashMap;
use std::io::Read;

use crate::parser;
use crate::ast;
use crate::builtins;
use crate::typecheck;
use crate::typecontext::TypeContext;

use ast::TermNode;
use ast::Def;
use ast::Import;
use ast::MatchArm;
use ast::Variable;
use ast::Term;
use builtins::TypeDef;

use super::value::Value;
use super::context::Context;
use super::import::ImportResolver;

///
/// Runtime is the global store for all of the information loaded into the program.
///
pub struct Runtime {
    /// Keeps track of which modules have been loaded into the Runtime
    /// already. This is currently being used to break cyclic imports.
    pub imports: Vec<String>,

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

        Runtime {
            imports: vec![],

            inductive_typedefs,

            definition_ctx: Context::empty(),
            builtin_ctx,

            definition_type_ctx: TypeContext::empty(),
            builtin_type_ctx,
        }
    }

    pub fn import(
        &mut self,
        import_name: &str,
        resolver: &mut dyn ImportResolver,
        is_main: bool,
    ) -> Result<(), RuntimeError> {
        let mut module_text = String::new();

        let mut resolved_import = resolver.resolve(import_name)?;
        resolved_import.reader.read_to_string(&mut module_text)?;
        let source = Some(resolved_import.source);

        let module = parser::parse_module(source, &module_text)?;

        for import in module.imports {
            let Import(name) = import;
            self.import(&name, resolver, false)?;
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

    pub fn exec(&mut self) {
        self.definition_ctx.lookup("main", 0).expect("There should be a main in your module");
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

    fn eval_variable(&self, v: &Variable, ctx: Context) -> Option<Value> {
        let x = &v.name;
        let k = v.layer;

        ctx.lookup(&x, k)
            .or_else(|| self.definition_ctx.lookup(x, k))
            .or_else(|| self.builtin_ctx.lookup(x, k))
    }

    pub fn eval_match(&mut self, t: &TermNode, match_arms: &[MatchArm], ctx: Context) -> Value {
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
    }

    pub fn eval_app(&mut self, f: &Term, vs: &[Term], ctx: Context) -> Value {
        let f_value = self.eval(&f, ctx.clone());
        let f_value = self.force(&f_value);
        let vs_values: Vec<Value> = vs.iter()
            .map(|v| Value::Thunk(v.clone(), ctx.clone()))
            .collect();
        self.apply(f_value, vs_values)
    }

    pub fn eval_let(&mut self, x: &str, v: &Term, body: &Term, ctx: Context) -> Value {
        let v_value = self.eval(&v, ctx.clone());
        let extended_ctx = ctx.extend(x, v_value);
        self.eval(&body, extended_ctx)
    }

    /// Evaluates a term in a given local context and returns the result.
    pub fn eval(&mut self, t: &TermNode, ctx: Context) -> Value {
        match t {
            TermNode::Var(v) => self.eval_variable(v, ctx).expect(&format!("Unbound variable {:?}", v)),
            TermNode::StrLit(contents) => Value::Str(contents.to_string()),
            TermNode::Hole(_hole_info) => unimplemented!(),
            TermNode::As(term, _typ) => self.eval(&term, ctx),
            TermNode::Match(t, match_arms) => self.eval_match(t, match_arms, ctx),
            TermNode::Lam(x, body) => Value::Fun(x.clone(), body.clone(), ctx.clone()),
            TermNode::App(f, vs) => self.eval_app(f, vs.as_slice(), ctx),
            TermNode::Let(x, v, body) => self.eval_let(x, v, body, ctx),
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
