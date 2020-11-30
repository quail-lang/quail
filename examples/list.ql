import nat
import pair

def head : List -> Nat = fun xs =>
    match xs
        with nil => ?{head of empty list}
        with cons x xs' => x

def tail : List -> List = fun xs =>
    match xs
        with nil => ?{tail of empty list}
        with cons x xs' => xs'

def length : List -> Nat = fun xs =>
    match xs
        with nil => zero
        with cons x xs' => succ (length xs')

def map : (Nat -> Nat) -> List -> List = fun f xs =>
    match xs
        with nil => nil
        with cons x xs' => cons (f x) (map f xs')

def filter : (Nat -> Bool) -> List -> List = fun p xs =>
    match xs
        with nil => nil
        with cons x xs' => (
            match p x
                with true => cons x (filter p xs')
                with false => filter p xs'
        )

def fold : Nat -> (Nat -> Nat -> Nat) -> List -> Nat = fun z s xs =>
    match xs
        with nil => z
        with cons x xs' => s z (fold z s xs')

def up_to_iter : Nat -> Nat -> List = fun n k =>
    match k
        with zero => nil
        with succ k' =>
            let r = sub n k
            in cons r (up_to_iter n k')

def up_to : Nat -> List = fun n => up_to_iter n n

def take : Nat -> List -> List = fun n xs =>
    match n
        with zero => nil
        with succ n' => (
            match xs
                with nil => nil
                with cons x xs' => cons x (take n' xs)
        )

def one_two_three : List =
    cons one (cons two (cons three nil))

def main : Top = println (show (length (one_two_three)))
