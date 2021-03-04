use crate::span::Span;
use super::Spanned;
use super::tokens;

fn merge_spans(spans: &[Span]) -> Span {
    let mut merged = spans[0];
    for span in spans[1..].iter().cloned() {
        merged = merged.merge(span);
    }
    merged
}

macro_rules! replace_ident { ($t:tt, $i:ident) => { $i } }

// Takes struct definitions and 
macro_rules! default_ast_impls {
    () => {};
    (pub struct $name:ident { $(pub $field:ident: $ty:ty,)+ } $($rest:tt)*) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $ty),+
        }

        impl $name {
            pub fn new($($field: $ty),+) -> $name {
                $name { $($field),+ }
            }
        }

        impl Spanned for $name {
            fn span(&self) -> Span {
                merge_spans(&[$(self.$field.span()),+])
            }
        }

        default_ast_impls! { $($rest)* }
    };

    //                                               vvvvvvvvvvvvv optional second type
    (pub enum $name:ident { $($variant:ident ($ty1:ty$(, $ty2:ty)?),)+ } $($rest:tt)*) => {
        #[derive(Debug, Clone)]
        pub enum $name {
            $($variant($ty1 $(, $ty2)?),)+
        }

        impl Spanned for $name {
            fn span(&self) -> Span {
                match self {
                    $($name::$variant(arg1 $(, replace_ident!($ty2, arg2))?) =>
                        merge_spans(&[arg1.span() $(, replace_ident!($ty2, arg2).span())?]),
                    )+
                }
            }
        }

        default_ast_impls! { $($rest)* }
    };
}

// #[derive(Debug, Clone)]

default_ast_impls! {
    pub struct Program {
        pub stmts: Vec<Stmt>,
    }

    pub enum Stmt {
        Macro(Macro),
        Expr(Expr),
    }

    pub struct Macro {
        pub name: tokens::Var,
        pub eq_token: tokens::Equal,
        pub value: Expr,
    }

    pub enum Expr {
        Lambda(Lambda),
        Close(Close),
        Appl(Appl),
    }

    pub struct Lambda {
        pub lambda_token: tokens::Lambda,
        pub var: tokens::Var,
        pub dot_token: tokens::Dot,
        pub expr: Box<Expr>,
    }

    pub struct Appl {
        pub lhs: Close,
        pub rhs: Close,
    }

    pub enum Close {
        Grouping(Box<Expr>, tokens::Group),
        Var(tokens::Var),
        Literal(tokens::Literal),
    }

    pub struct VarList {
        pub list: Vec<(tokens::Var, tokens::Dot)>,
    }
}


