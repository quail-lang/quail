#![allow(dead_code, unused_variables)]
use std::collections::HashSet;

use crate::ast as q;
use super::ast as m;

pub fn transform(module: q::Module) -> m::Program {
    assert!(module.imports.is_empty()); // TODO deal with non-empty imports later

    let mut bindings = vec![];

    for q::Def(var, typ, term) in &module.definitions {
        let e = transform_term(term.clone(), &module);
        let updatable = var == "main";
        let lf = m::LambdaForm(vec![], updatable, vec![], e);
        bindings.push(m::Binding(var.clone(), lf));
    }

    let ctors = [
        ("zero", vec![]),
        ("succ", vec!["n"]),
        ("nil",  vec![]),
        ("cons", vec!["x", "xs"]),
        ("true",  vec![]),
        ("false",  vec![]),
    ];

    for (ref name, args) in ctors.iter() {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let args_atoms: Vec<m::Atom> = args.iter().map(|s| m::Atom::Var(s.to_string())).collect();
        let e = m::ExprNode::App(m::AppType::Ctor, name.to_string(), args_atoms).into();
        let lf = m::LambdaForm(vec![], false, args, e);
        bindings.push(m::Binding(name.to_string(), lf));
    }

    m::Program(bindings)
}

fn transform_term_let(x: &str, s: q::Term, t: q::Term, program: &q::Module) -> m::Expr {
    let mut free_vars = gather_free_vars(s.clone(), program);
    free_vars.retain(|fv| fv != x);

    m::ExprNode::Let(
        m::LetType::NonRecursive,
        vec![
            m::Binding(
                x.to_owned(),
                m::LambdaForm(
                    free_vars,
                    decide_updatability(),
                    vec![], // TODO should this be fused if the target is a lambda?
                    transform_term(s.clone(), program),
                )
            ),
        ],
        transform_term(t.clone(), program),
    ).into()
}

fn transform_term_lam(x: &str, t: q::Term, program: &q::Module) -> m::Expr {
    let gensym = "gensym_0".to_owned();
    let mut free_vars = gather_free_vars(t.clone(), program);
    free_vars.retain(|fv| fv != &x);

    m::ExprNode::Let(
        m::LetType::NonRecursive,
        vec![m::Binding(
            gensym.clone(),
            m::LambdaForm(
                free_vars,
                false,
                vec![x.to_owned()],
                transform_term(t.clone(), program),
            ),
        )],
        m::ExprNode::App(m::AppType::Fun, gensym, vec![]).into(),
    ).into()
}

fn transform_term_app(t: q::Term, vs: &[q::Term], program: &q::Module) -> m::Expr {
    let mut i = 0;
    let mut temps = vec![];

    let f = get_var_name(t.clone()).unwrap_or_else(|| {
        let name = format!("gensym_{}", i);
        temps.push((name.clone(), t.clone()));
        i += 1;
        name
    });

    let vs_expr = vs.iter().map(|v| {
        m::Atom::Var(get_var_name(v.clone()).unwrap_or_else(|| {
            let name = format!("gensym_{}", i);
            temps.push((name.clone(), v.clone()));
            i += 1;
            name
        }))
    }).collect();

    let app_type = if is_ctor(&f) {
        m::AppType::Ctor
    } else {
        m::AppType::Fun
    };

    dbg!(&app_type);

    if temps.is_empty() {
        m::ExprNode::App(app_type, f, vs_expr).into()
    } else {
        let mut bindings = vec![];
        for (name, term) in temps.iter() {
            bindings.push(m::Binding(
                    name.clone(),
                    m::LambdaForm(
                        gather_free_vars(term.clone(), program), //xs
                        decide_updatability(),
                        vec![], // TODO should this be fused if the target is a lambda?
                        transform_term(term.clone(), program),
                    )
            ))
        }

        m::ExprNode::Let(
            m::LetType::NonRecursive,
            bindings,
            m::ExprNode::App(app_type, f, vs_expr).into(),
        ).into()
    }
}

fn is_ctor(var: &str) -> bool {
    vec![
        "succ",
        "zero",
        "true",
        "false",
        "nil",
        "cons",
    ].contains(&var)
}

fn transform_term_match(t: q::Term, match_arms: &[q::MatchArm], program: &q::Module) -> m::Expr {
    let t_expr = transform_term(t.clone(), program);
    let arm_exprs = m::Alts(match_arms.iter().map(|q::MatchArm(pat, s)| {
        let (ctor, xs) = pat.split_first().unwrap(); // TODO
        let xs = xs.iter().map(|x| x.to_owned()).collect();
        let s_expr = transform_term(s.clone(), program);
        m::Alt::Ctor(ctor.clone(), xs, s_expr)
    }).collect());

    m::ExprNode::Case(
        t_expr,
        arm_exprs,
    ).into()
}

fn transform_term_var(var: &q::Variable, program: &q::Module) -> m::Expr {
    dbg!(&var.name);
    assert_eq!(var.layer, 0); // TODO deal with this
    if is_ctor(&var.name) {
        m::ExprNode::App(m::AppType::Ctor, var.name.clone(), vec![]).into()
    } else {
        m::ExprNode::App(m::AppType::Fun, var.name.clone(), vec![]).into()
    }
}

fn transform_term(term: q::Term, program: &q::Module) -> m::Expr {
    use q::TermNode::*;

    match term.as_ref() {
        As(t, typ) => transform_term(t.clone(), program),
        StrLit(_s) => todo!(),
        Hole(_s) => todo!(),
        Let(x, s, t) => transform_term_let(x, s.clone(), t.clone(), program),
        Var(var) => transform_term_var(var, program),
        Lam(x, t) => transform_term_lam(&x, t.clone(), program),
        App(t, vs) => transform_term_app(t.clone(), vs, program),
        Match(t, match_arms) => transform_term_match(t.clone(), &match_arms, program),
    }
}

fn get_var_name(t: q::Term) -> Option<String> {
    if let q::TermNode::Var(v) = t.as_ref() {
        Some(v.name.clone())
    } else {
        None
    }
}

fn decide_updatability() -> bool {
    true
}

fn top_level_vars(module: &q::Module) -> Vec<m::Var> {
    let mut vars = vec![];
    for q::Def(var, _typ, _term) in module.definitions.iter() {
        vars.push(var.clone());
    }
    vars
}

fn gather_free_vars(t: q::Term, module: &q::Module) -> Vec<m::Var> {
    let free_vars = t.free_vars().iter().map(|v| v.name.clone()).collect::<HashSet<String>>();
    let top_level_vars = top_level_vars(module).iter().map(|s| s.to_owned()).collect::<HashSet<String>>();
    let ctors: HashSet<String> = [
        "zero".to_owned(),
        "succ".to_owned(),
        "nil".to_owned(),
        "cons".to_owned(),
        "true".to_owned(),
        "false".to_owned(),
    ].iter().cloned().collect::<HashSet<_>>();

    let free_vars: Vec<m::Var> = free_vars.difference(&top_level_vars).cloned().collect::<HashSet<_>>().difference(&ctors).cloned().collect::<Vec<_>>();
    assert!(!free_vars.contains(&"succ".to_owned()));
    free_vars
}
