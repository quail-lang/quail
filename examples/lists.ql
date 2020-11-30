def one = succ zero
def two = succ one
def three = succ two
def four = add two two
def five = succ four

def main = println (cons one (cons two (cons three nil)))
