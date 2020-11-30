import nat
import list
import pair

def fib_iter = fun n a b =>
    match n
        with zero => add a b
        with succ n' => fib_iter n' b (add a b)

def fib = fun n =>
    match n
        with zero => zero
        with succ n' => (
            match n'
                with zero => succ zero
                with succ n'' => fib_iter n'' zero (succ zero)
            )

def main =
    map (fun n => println (fib n)) (up_to ten)
