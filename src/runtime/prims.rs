use super::{Runtime, Value};

pub(super) fn cat(_runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 2, "show_list must have exactly two arguments");
    let v1 = vs[0].clone();
    let v2 = vs[1].clone();
    match (&v1, &v2) {
        (Value::Str(s1), Value::Str(s2)) => Value::Str(format!("{}{}", s1, s2)),
        _ => panic!("Arguments to cat must both be Str: {:?} {:?}", &v1, &v2),
    }
}

pub(super) fn println(_runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "println must have exactly one argument");
    let v = vs[0].clone();
    println!("{:?}", v);
    Value::Ctor("unit".into(), Vec::new())
}

pub(super) fn show(runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "show must have exactly one argument");
    let v = vs[0].clone();
    match &v {
        Value::Ctor(tag, _) => {
            if tag == "zero" || tag == "succ" {
                Value::Str(format!("{}", nat_to_u64(v)))
            } else if tag == "nil" || tag == "cons" {
                let val_vec = list_to_vec(v.clone());
                let str_value_vec: Vec<Value> = val_vec.into_iter()
                    .map(|v| show(runtime, vec![v]))
                    .collect();
                let s: String = format!("{:?}", str_value_vec);
                Value::Str(s)
            } else {
                Value::Str(format!("{:?}", v))
            }
        },
        _ => panic!("Can't show this {:?}", &v),
    }
}

pub(super) fn show_list(runtime: &mut Runtime, vs: Vec<Value>) -> Value {
    assert_eq!(vs.len(), 1, "show_list must have exactly one argument");
    let v = vs[0].clone();
    match &v {
        Value::Ctor(tag, _) => {
            if tag == "zero" || tag == "succ" {
                Value::Str(format!("{}", nat_to_u64(v)))
            } else if tag == "nil" || tag == "cons" {
                let val_vec = list_to_vec(v.clone());
                let str_value_vec: Vec<Value> = val_vec.into_iter()
                    .map(|v| show(runtime, vec![v]))
                    .collect();
                let s: String = format!("{:?}", str_value_vec);
                Value::Str(s)
            } else {
                Value::Str(format!("{:?}", v))
            }
        },
        _ => panic!("Can't show this {:?}", &v),
    }
}

fn list_to_vec(v: Value) -> Vec<Value> {
    match v {
        Value::Ctor(tag, contents) => {
            if tag == "nil" {
                Vec::new()
            } else if tag == "cons" {
                let head = &contents[0];
                let tail = &contents[1];
                let mut result = list_to_vec(tail.clone());
                result.insert(0, head.clone());
                result
            } else {
                panic!("This isn't a list.")
            }
        },
        _ => panic!("This isn't a list."),
    }
}

fn nat_to_u64(v: Value) -> u64 {
    let mut val = v;
    let mut result = 0;

    loop {
        match val {
            Value::Ctor(tag, contents) => {
                if tag == "zero" {
                    break
                } else if tag == "succ" {
                    val = contents[0].clone();
                    result += 1;
                } else {
                    panic!("This isn't a nat.")
                }
            },
            _ => panic!("This isn't a nat."),
        }
    }

    result
}