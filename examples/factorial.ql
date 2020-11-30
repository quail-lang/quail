def add = fun n m =>
    (ifzero n
        (fun x => m)
        (fun x => (add (pred n) (succ m)))) 0


def mul = fun n m =>
    (ifzero n
        (fun x => 0)
        (fun x => (add m (mul (pred n) m)))) 0

def factorial = fun n =>
    (ifzero n
        (fun x => 1)
        (fun x => (mul n (factorial (pred n))))) 0

def main = println (factorial 5)
