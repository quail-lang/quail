use super::ast::*;
use super::*;
use super::heap::{heap_to_string};
use super::machine::{Addr, Closure, Value, Context};

#[test]
fn test_stg_works() {
    let xs = LambdaForm(vec![], false, vec![],
        ExprNode::App(
            AppType::Ctor,
            "Nil".to_owned(),
            vec![],
        ).into(),
    );

    let pure = LambdaForm(vec![], false, vec!["a".to_owned()],
        ExprNode::App(
            AppType::Ctor,
            "Cons".to_owned(),
            vec!["a".into(), "xs".into()],
        ).into(),
    );

    let map = LambdaForm(vec![], false, vec!["f".to_owned(), "xs".to_owned()],
        ExprNode::Case(
            ExprNode::App(AppType::Fun, "xs".to_owned(), vec![]).into(),
            Alts(vec![
                Alt::Ctor(
                    "Nil".to_owned(),
                    vec![],
                    ExprNode::App(AppType::Fun, "Nil".to_owned(), vec![]).into(),
                ),
                Alt::Ctor(
                    "Cons".to_owned(),
                    vec!["y".to_owned(), "ys".to_owned()],
                    ExprNode::Let(
                        LetType::NonRecursive,
                        vec![
                            Binding(
                                "fy".to_owned(),
                                LambdaForm(
                                    vec!["f".to_owned(), "y".to_owned()],
                                    true,
                                    vec![],
                                    ExprNode::App(AppType::Fun, "f".into(), vec!["y".into()]).into()
                                ),
                            ),
                            Binding(
                                "mfy".to_owned(),
                                LambdaForm(
                                    vec!["f".to_owned(), "ys".to_owned()],
                                    true,
                                    vec![],
                                    ExprNode::App(AppType::Fun, "map".into(), vec!["f".into(), "y".into()]).into(),
                                ),
                            ),
                        ],
                        ExprNode::App(AppType::Ctor, "Cons".to_owned(), vec!["fy".into(), "mfy".into()]).into(),
                    ).into(),
                ),
            ]),
        ).into()
    );

    // TODO pretty print exprs to make things readable.
    let zeroes = LambdaForm(vec![], false, vec!["n".to_owned()],
        ExprNode::Case(
            ExprNode::App(AppType::Fun, "n".to_owned(), vec![]).into(),
            Alts(vec![
                Alt::Lit(
                    0,
                    ExprNode::App(AppType::Ctor, "Nil".to_owned(), vec![]).into()
                ),
                Alt::Default(
                    "TODO-NOBIND".to_owned(),
                    ExprNode::Case(
                        ExprNode::App(AppType::Prim, "-1".to_owned(), vec![Atom::Var("n".to_owned())]).into(),
                        Alts(vec![
                            Alt::Default(
                                "m".to_owned(),
                                ExprNode::Let(
                                    LetType::NonRecursive,
                                    vec![
                                        Binding(
                                            "tail".to_owned(),
                                            LambdaForm(vec!["m".to_owned()], true, vec![],
                                                 ExprNode::App(AppType::Fun, "zeroes".to_owned(), vec![Atom::Var("m".to_owned())]).into())
                                        ),
                                    ],
                                    ExprNode::App(AppType::Ctor, "Cons".to_owned(), vec![Atom::Lit(0), Atom::Var("tail".to_owned())]).into()
                                ).into()
                            ),
                        ]),
                    ).into()
                ),
            ]),
        ).into()
    );

    #[allow(unused_variables)]
    let example1 = ExprNode::App(
        AppType::Fun,
        "pure".to_owned(),
        vec![Atom::Lit(3)],
    );

    #[allow(unused_variables)]
    let example2 = ExprNode::App(
        AppType::Prim,
        "+".to_owned(),
        vec![Atom::Lit(2), Atom::Lit(3)],
    );

    #[allow(unused_variables)]
    let example3 = ExprNode::App(
        AppType::Fun,
        "xs".to_owned(),
        vec![],
    );

    #[allow(unused_variables)]
    let example4 = ExprNode::App(
        AppType::Fun,
        "zeroes".to_owned(),
        vec![Atom::Lit(3)],
    );

    let main = example4;

    let bindings = vec![
        Binding("xs".to_owned(), xs),
        Binding("pure".to_owned(), pure),
        Binding("zeroes".to_owned(), zeroes),
        Binding("map".to_owned(), map),
        Binding("main".to_owned(), LambdaForm(vec![], true, vec![], main.into())),
    ];

    let program = Program(bindings);

    let mut m = StgMachine::new(&program, Some("main"));

    while !m.is_halted() {
        m.step();
    }
    eprintln!("HEAP:");
    eprintln!("{}", heap_to_string(&m.heap));

    eprintln!("#######################################################");
    eprintln!("DEEP SEQUING!");
    eprintln!("#######################################################");

    m.deep_seq(5);
    eprintln!("HEAP:");
    eprintln!("{}", heap_to_string(&m.heap));
    eprintln!("GLOBALS:");
    for g in m.globals.iter() {
        eprintln!("    {:?}", g);
    }

    let v = m.lookup_global_addr("main").unwrap();
    print_stuff(&m, v);
    println!();

    assert!(true);
}

fn print_stuff(m: &StgMachine, addr: Addr) {
    let heap = &m.heap;
    let Closure(LambdaForm(vs, _pi, xs, e), ws) = heap.lookup(addr);
    let ctx = Context::from(vs.clone(), ws.clone());

    if xs.len() > 0 {
        println!("<THUNK>");
    } else {
        if let ExprNode::App(AppType::Ctor, f, ts) = e.as_ref() {
            print!("{}", f);
            for (i, t) in ts.iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                match t {
                    Atom::Lit(k) => print!(" {}", k),
                    Atom::Var(name) => {
                        let value = m.lookup_var(name, &ctx);
                        match value {
                            Value::Addr(a) => {
                                print!(" (");
                                print_stuff(m, a);
                                print!(")");
                            },
                            Value::Int(k) => print!(" {}", k),
                        }
                    },
                }
            }
        } else {
            dbg!(&e.as_ref());
            panic!();
        }
    }
}

use crate::parser::*;
use crate::resolver::FileImportResolver;
use crate::resolver::ImportResolver;

use super::transform::transform;

#[test]
fn test_transform() {
    let mut import_resolver = FileImportResolver::new("stg_examples");
    let text = import_resolver.resolve("nat").unwrap().text();
    let module = parse_module(None, &text).unwrap();
    let program = transform(module);

    eprintln!("{}", program);
}


#[test]
fn test_transform_run() {
    let mut import_resolver = FileImportResolver::new("stg_examples");
    let text = import_resolver.resolve("nat").unwrap().text();
    let module = parse_module(None, &text).unwrap();
    let program = transform(module);
    let mut m = StgMachine::new(&program, Some("main"));
    println!("********************************************************************************");
    println!("PROGRAM:");
    println!("{}", program);
    eprintln!("GLOBALS:");
    for g in m.globals.iter() {
        eprintln!("    {:?}", g);
    }

    while !m.is_halted() {
        m.step();
    }

    eprintln!("GLOBALS:");
    for g in m.globals.iter() {
        eprintln!("    {:?}", g);
    }

    eprintln!("HEAP:");
    eprintln!("{}", heap_to_string(&m.heap));
    let main_addr = m.lookup_global_addr("main").unwrap();
    eprintln!("Main is at {}", main_addr);

    eprintln!("#######################################################");
    eprintln!("DEEP SEQUING!");
    eprintln!("#######################################################");
    m.deep_seq(main_addr);
    eprintln!("HEAP:");
    eprintln!("{}", heap_to_string(&m.heap));
    eprintln!("GLOBALS:");
    for g in m.globals.iter() {
        eprintln!("    {:?}", g);
    }

    dbg!(&main_addr);
    println!("##########");
    print_stuff(&m, main_addr);
    println!();
    println!("##########");
}
