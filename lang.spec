
Goal    ::= Prog $
Prog    ::= Stmt*
Stmt    ::= Macro | Expr
Macro   ::= Var Space Equal Space Expr
Expr    ::= Lambda | Close | Appl | Literal
Lambda  ::= λ Var Dot Space Expr
Appl    ::= Close Space Close
Close   ::= OParen Expr CParen | Var
