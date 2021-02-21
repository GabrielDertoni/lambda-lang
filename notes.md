
```text
a. b. a (c. b c)

expr1: a. expr2
expr2: b. a expr3
expr3: c. b c

expr1: a. expr2
expr2: a. b. a expr3
expr3: a. b. c. b c

expr1: <a> -> expr2<a>
expr2: <a, b> -> a expr3<b>
expr3: <b, c> -> b c



a. (b. a (b b)) (b. a (b b))

expr1: <a> -> expr2<a> expr2<a>
expr2: <a, b> -> a expr3<b>
expr3: <b> -> b b

expr1: a. ((b. c. b ((d. d d) c)) a) ((b. c. b ((d. d d) c)) a)
expr2: a. b. a ((c. c c) b)
expr3: b. b b

expr4: <a> -> a

expr1<expr4> -> expr2<expr4, expr2<expr4>>
expr2<expr4, expr2<expr4>> -> expr4<expr3<expr2<expr4>>>
expr4<expr3<expr2<expr4>>> -> expr3<expr2<expr4>>>
expr2<expr4, expr2<expr4>> -> expr3<expr2<expr4>>
expr3<expr2<expr4>> -> expr2<expr4, expr2<expr4>>

Y   = a. (b. a (b b)) (b. a (b b))
Y g = (a. (b. a (b b)) (b. a (b b))) g
Y g = (b. g (b b)) (b. g (b b))
Y g = g ((b. g (b b)) (b. g (b b)))
Y g = g (Y g)

g. (a. (b. a (b b)) (b. a (b b))) g = g. g ((a. (b. a (b b)) (b. a (b b))) g)

expr1: <a> -> (b. a (b b)) (b. a (b b))
expr2: <>  -> a ((b. a (b b)) (b. a (b b)))
expr2: <>  -> a<expr1<a>>

expr1: <a, b> -> expr2<a, expr2<a, b>>
expr2: <a, b> -> a<b<b>>

expr2: <a, b> -> a<expr2<a, b>>


expr1<a>: a<expr1<a>>


```

```rust
enum LambdaExpr {
    Application {
        f: i32,
        params: Box<[i32]>,
    },
    Itself,
}

struct Lambda {
    params: Box<[i32]>,
    expr: LambdaExpr,
}

impl Lambda {
    fn eval(&self) -> Expr {
    }
}
```

```txt
Prog    ::= Line\nProg | Line
Line    ::= Macro | Expr | \n
Expr    ::= Lambda | Close | Appl | Literal
Macro   ::= Var = Expr
Lambda  ::= Var. Expr
Appl    ::= Close Close
Close   ::= (Expr) | Var


Goal    -> Line $
Line    -> Macro
Line    -> Expr
Macro   -> Var = Expr
Expr    -> Lambda
Expr    -> Close
Expr    -> Appl
Expr    -> Literal
Lambda  -> Var. Expr
Appl    -> Close Close
Close   -> (Expr)
Close   -> Var

Item set 0:
===========
Goal   -> _Line $
Line   -> _Macro
Line   -> _Expr
Macro  -> _Var = Expr
Expr   -> _Lambda
Expr   -> _Close
Expr   -> _Appl
Expr   -> _Literal
Lambda -> _λVar. Expr
Appl   -> _Close Close
Close  -> _(Expr)
Close  -> _Var

Item set 1 (from 0, x = λ):
===========================
Lambda -> λ_Var. Expr

Item set 1 (from 1, x = Var):
===========================
Lambda -> λVar_. Expr

Item set 1 (from 0, x = Var):
=============================
Close  -> Var_

Item set 2 (from 0, x = '('):
=============================
Close  -> (_Expr)
Expr   -> _Lambda
Expr   -> _Close
Expr   -> _Appl
Expr   -> _Literal
Lambda -> _Var. Expr
Close  -> _(Expr)
Close  -> _Var
Appl   -> _Close Close

Item set 3 (from 0, x = Close):
===============================
Expr   -> Close_
Appl   -> Close_ Close

Item set 4 (from 0, x = Literal):
=================================
Expr   -> Literal_

Item set 5 (from 0, x = Appl):
==============================
Expr   -> Appl_

Item set 6 (from 0, x = Lambda):
================================
Expr   -> Lambda_

Item set 7 (from 0, x = Expr):
==============================
Line   -> Expr_

Item set 8 (from 0, x = Macro):
===============================
Macro  -> Var_ = Expr

Item set 9 (from 0, x = Line):
==============================
Goal  -> Line_$

Item set 10 (from set 1, x = Var):
==================================
Lambda -> Var_. Expr

Item set 11 (from set 2, x = Expr):
===================================
Close  -> (Expr_)

Item set 12 (from set 3, x = ' '):
==================================
Appl   -> Close _Close

Item set 13 (from set 8, x = ' '):
==================================
Macro  -> Var _= Expr

Item set 14 (from set 10, x = '.'):
===================================
Lambda -> Var._ Expr

Item set 15 (from set 11, x = ')'):
===================================
Close  -> (Expr)_

Item set 16 (from set 12, x = Close):
=====================================
Appl   -> Close Close_

Item set 17 (from set 13, x = '='):
===================================
Macro  -> Var =_ Expr

Item set 18 (from set 14, x = ' '):
===================================
Lambda -> Var. _Expr
Expr   -> _Lambda
Expr   -> _Close
Expr   -> _Appl
Expr   -> _Literal
Lambda -> _Var. Expr
Close  -> _(Expr)
Close  -> _Var
Appl   -> _Close Close

Item set 19 (from set 17, x = ' '):
===================================
Macro  -> Var = _Expr
Expr   -> _Lambda
Expr   -> _Close
Expr   -> _Appl
Expr   -> _Literal
Lambda -> _Var. Expr
Close  -> _(Expr)
Close  -> _Var
Appl   -> _Close Close

Item set 20 (from set 18, x = Expr):
====================================
Lambda -> Var. Expr_

Item set 21 (from set 19, x = Expr):
====================================
Macro  -> Var = Expr_


Table:
======

| Item set | ' ' | '.' | '(' | ')' | '=' | Literal | Var | Line | Expr | Macro | Lambda | Appl | Close |
|----------|-----|-----|-----|-----|-----|---------|-----|------|------|-------|--------|------|-------|

| Item set | ' ' | '.' | '(' | ')' | '=' | Literal | Var | Line | Expr | Macro | Lambda | Appl | Close |
| 0        |     |     | 2   |     |     | 4       | 1   | 9    | 7    | 8     | 6      | 5    | 3     |
| 1        |     |     |     |     |     |         | 10  |      |      |       |        |      |       |
| 2        |     |     | 2   |     |     | 4       | 10  |      | 11   |       | 6      | 5    | 3     |
| 3        |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 4        |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 5        |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 6        |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 7        |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 8        |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 9        |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 10       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 11       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 12       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 13       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 14       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 15       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 16       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 17       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 18       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 19       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 20       |     |     |     |     |     |         |     |      |      |       |        |      |       |
| 21       |     |     |     |     |     |         |     |      |      |       |        |      |       |

C = x. y. x y

**Lexer**
Var = Var. Var. Var Var

**Parser**
Macro
Var = Expr
Var = Var. Expr
Var = Var. Var. Expr
Var = Var. Var. Appl
Var = Var. Var. Close Close
Var = Var. Var. Var Var

Y = f. (x. f (x x)) (x. f (x x))

**Lexer**
Var = Var. (Var. Var (Var Var)) (Var. Var (Var Var))

**Parser**
Macro
Var = Expr
Var = Var. Expr
Var = Var. Appl
Var = Var. Close Close
Var = Var. (Expr) (Expr)
Var = var. (Lambda
```
