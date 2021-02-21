use super::tokens;

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Macro(Macro),
    Expr(Expr),
}

#[derive(Debug)]
pub struct Macro {
    pub name: tokens::Var,
    pub eq_token: tokens::Equal,
    pub value: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Lambda(Lambda),
    Close(Close),
    Appl(Appl),
    Literal(tokens::Literal),
}

#[derive(Debug)]
pub struct Lambda {
    pub lambda_token: tokens::Lambda,
    pub var: tokens::Var,
    pub dot_token: tokens::Dot,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub enum Close {
    Paren(Box<Expr>),
    Var(tokens::Var),
}

#[derive(Debug)]
pub struct Appl {
    pub rhs: Close,
    pub lhs: Close,
}



