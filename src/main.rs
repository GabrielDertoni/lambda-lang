#![allow(dead_code)]
#![feature(array_windows)]
#![feature(try_trait)]
#![feature(hash_set_entry)]
#![feature(str_split_once)]
#![feature(box_patterns)]
#![feature(bindings_after_at)]

use std::collections::{ HashMap, HashSet };
use rustyline::error::ReadlineError;
use rustyline::Editor;

mod interpreter;
mod compiler;
mod parser;
mod utils;

use crate::compiler::{ compile_stmt, StmtReturn };
// use crate::interpreter::print_expr;

fn main() -> std::io::Result<()> {
    // let args: Vec<String> = std::env::args().collect();

    let mut literals = HashSet::new();
    let mut macros = HashMap::new();

    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if line == "exit" { break; }
                match compile_stmt(line.as_str(), &mut literals, &mut macros) {
                    Ok(StmtReturn::Macro(name))    => println!("Defined macro {}", name),
                    Ok(StmtReturn::Expr(mut expr)) => {
                        match expr.eval() {
                            Ok(res)  => println!("{}", res),
                            Err(err) => println!("RuntimeError:\n\t{}", err),
                        }
                    },
                    Err(err) => println!("{:?}", err),
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    /*
    let fname = &args[1];
    let content = std::fs::read_to_string(fname)?;

    let mut mem_static = Vec::new();

    let mut compiled = match compile_program(&content, &mut mem_static) {
        Ok(comp) => comp,
        Err(err) => {
            println!("{}", err);
            return Ok(());
        },
    };

    match compiled.eval() {
        Ok(res)  => println!("{:#?}", res),
        Err(err) => println!("RuntimeError:\n\t{}", err),
    }
    */

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::compiler::compile_program;

    #[test]
    fn test_infinite_loop() {
        let input = "(\\a. a a) (\\a. a a)";
        let mut compiled = compile_program(input).unwrap();
        assert!(compiled.eval().is_err());
    }

    #[test]
    fn test_id() {
        let input = "(\\a. a) \"hello\"";
        let mut compiled = compile_program(input).unwrap();
        println!("{:?}", compiled.eval().unwrap());
    }
}
