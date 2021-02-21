#![allow(dead_code)]
#![feature(array_windows)]
#![feature(try_trait)]
#![feature(hash_set_entry)]
#![feature(str_split_once)]

mod interpreter;
mod compiler;
mod parser;
mod utils;

use crate::interpreter::*;
use crate::compiler::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let fname = &args[1];
    let content = std::fs::read_to_string(fname)?;

    // (a. a a) (a. a a)
    let mut expr2 = Expr::Appl {
        f: Expr::Lambda {
            param: 0,
            expr: Expr::Appl {
                f: Expr::Var( 0 ).into(),
                arg: Expr::Var( 0 ).into(),
            }.into()
        }.into(),
        arg: Expr::Lambda {
            param: 0,
            expr: Expr::Appl {
                f: Expr::Var( 0 ).into(),
                arg: Expr::Var( 0 ).into(),
            }.into()
        }.into(),
    };

    // (a. a a) ((a. a a) (a. a a))
    let mut expr3 = Expr::Appl {
        f: Expr::Lambda {
            param: 0,
            expr: Expr::Appl {
                f: Expr::Var( 0 ).into(),
                arg: Expr::Var( 0 ).into(),
            }.into()
        }.into(),
        arg: Expr::Appl {
            f: Expr::Lambda {
                param: 0,
                expr: Expr::Appl {
                    f: Expr::Var( 0 ).into(),
                    arg: Expr::Var( 0 ).into(),
                }.into()
            }.into(),
            arg: Expr::Lambda {
                param: 0,
                expr: Expr::Appl {
                    f: Expr::Var( 0 ).into(),
                    arg: Expr::Var( 0 ).into(),
                }.into()
            }.into(),
        }.into(),
    };

    println!("Expression2 is {}", expr2);
    match expr2.eval() {
        Ok(res)  => println!("Evaluated2: {}", res),
        Err(err) => println!("Error2: {}", err),
    }

    println!("Expression3 is {}", expr3);
    match expr3.eval() {
        Ok(res)  => println!("Evaluated3: {}", res),
        Err(err) => println!("Error3: {}", err),
    }

    Ok(())
}

