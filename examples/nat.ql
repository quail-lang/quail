def one = succ zero
def two = succ one
def three = succ two
def four = succ three
def five = succ four
def six = succ five
def seven = succ six
def eight = succ seven
def nine = succ eight
def ten = succ nine

def add n m =
    match n
        with zero => m
        with succ n => succ (add n m)

def mul n m =
    match n
        with zero => zero
        with succ n => add m (mul n m)

def pow n m =
    match m
        with zero => succ zero
        with succ m' => mul n (pow n m')

def sub n m =
    match m
        with zero => n
        with succ m' => (
            match n
                with zero => zero
                with succ n' => sub n' m'
        )

def rem n m =
    match less_than n m
        with true => n
        with false => rem (sub n m) m

def eq n m =
    match n
        with zero => (
            match m
                with zero => true
                with succ x => false
        )
        with succ n' => (
            match m
                with zero => false
                with succ m' => eq n' m'
        )

def less_than_eq n m =
    match n
        with zero => true
        with succ n' => (
            match m
                with zero => false
                with succ m' => less_than_eq n' m'
        )

def less_than n m =
    less_than_eq (succ n) m

def main = println (eq zero zero)
