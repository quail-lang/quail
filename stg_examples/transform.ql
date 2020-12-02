def main : Top =
    let x = zero
    in x

def foo : Top = f x y

def bar : Top = f (g x)

def baz : Top =
    match zero
        with zero => zero
        with succ n => succ zero

def identity_top : Top -> Top = fun t =>
    match t
        with top => top
