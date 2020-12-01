use std::collections::HashSet;
use std::convert::TryInto;

use crate::ast;
use crate::parser::*;
use crate::resolver::FileImportResolver;
use crate::resolver::ImportResolver;

#[test]
fn free_vars_examples() {
    let mut import_resolver = FileImportResolver::new("examples");
    let text = import_resolver.resolve("hello").unwrap().text();
    let module = parse_module(None, &text).unwrap();
    let ast::Def(_name, _typ, body) = module.definition("main").unwrap();
    assert_eq!(body.free_vars(), vec!["println".try_into().unwrap()].into_iter().collect());
}


#[test]
fn free_vars() {
    macro_rules! assert_free_vars_in_term {
        ($text:expr, $vs:expr) => {
            let term = parse_term(None, &$text).unwrap();

            let mut frees: HashSet<ast::Variable> = HashSet::new();
            for v in ($vs as &[&str]) {
                let var = (*v).try_into().unwrap();
                frees.insert(var);
            }
            assert_eq!(term.free_vars(), frees);
        }
    }

    // bare variables (even with type annotations) are free
    assert_free_vars_in_term!("x", &["x"]);
    assert_free_vars_in_term!("x as Nat", &["x"]);

    // literals and holes have no free variables
    assert_free_vars_in_term!("\"hello\"", &[]);
    assert_free_vars_in_term!("?", &[]);
    // TODO assert_free_vars_in_term!("0", &[]);

    // applications have frees which are the union of the function and args
    assert_free_vars_in_term!("f x", &["f", "x"]);
    assert_free_vars_in_term!("f x y", &["f", "x", "y"]);

    // lets draw free vars from their definition and from the body,
    // albiet the let-bound variable is captured
    assert_free_vars_in_term!("let x = zero in x", &["zero"]);
    assert_free_vars_in_term!("let x = zero in y", &["zero", "y"]);
    assert_free_vars_in_term!("let x = ? in y", &["y"]);
    assert_free_vars_in_term!("let x = ? in x y", &["y"]);

    // functions have frees which are those of the body
    // modulo capture by the fun-binding
    assert_free_vars_in_term!("fun x => x y", &["y"]);
    assert_free_vars_in_term!("fun x y => x y", &[]);
    assert_free_vars_in_term!("fun x => fun y => x y", &[]);
    // TODO assert_free_vars_in_term!("fun => x", &["x"]);

    // matches
    assert_free_vars_in_term!("match x
                                with Zero => y
                                with Succ z => f z
                              ", &["x", "y", "f"]);
}
