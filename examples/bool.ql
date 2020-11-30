def not : Bool -> Bool = fun b =>
    match b
        with true => false
        with false => true

def and : Bool -> Bool -> Bool = fun a b =>
    match a
        with true => b
        with false => false

def or : Bool -> Bool -> Bool = fun a b =>
    match a
        with true => true
        with false => b

def main : Top =
    let v = (
        match and true true
            with true => succ zero
            with false => zero
    ) as Nat in println (show v)
