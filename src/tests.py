
func = lambda f: lambda x: lambda: f(x)

def evaluate(f, *args):
    i = 0
    while callable(f) and i < len(args):
        n = len(f.__code__.co_varnames)
        if n == 0:
            f = f()
        else:
            f = f(args[i])
            i += 1

    return f()

eval_n = lambda n: evaluate(n(func(lambda x: x + 1))(0))


zero = func(lambda f: func(lambda x:     x   ))
one  = func(lambda f: func(lambda x:   f(x)  ))
two  = func(lambda f: func(lambda x: f(f(x)) ))

succ = func(lambda n: func(lambda f: func(lambda x: f(n(f)(x)))))
pred = func(lambda n: func(lambda f: func(lambda x: n(func(lambda g: func(lambda h: h(g(f))))(func(lambda u: x))(func(lambda u: u))))))


prev = zero
for i in range(10):
    print(evaluate(prev, lambda x: x + 1, 0))
    prev = evaluate(succ, prev)





