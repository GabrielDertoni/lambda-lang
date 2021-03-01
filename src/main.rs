#![allow(dead_code)]
#![allow(incomplete_features)]
#![feature(array_windows)]
#![feature(try_trait)]
#![feature(hash_set_entry)]
#![feature(str_split_once)]
#![feature(box_patterns)]
#![feature(bindings_after_at)]
#![feature(if_let_guard)]

mod interpreter;
mod compiler;
mod parser;
mod utils;

// TODO: Maybe will became a submodule somewhere.
// mod thunk;

use std::collections::{ HashMap, HashSet };
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::compiler::{ compile_stmt, StmtReturn };

fn main() -> std::io::Result<()> {
    let mut literals = HashSet::new();
    let mut macros = HashMap::new();

    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".lambda");

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
                            Err(err) => {
                                eprintln!("RuntimeError:\n\t{}", err);
                                eprintln!("Error occurred at: {}", expr);
                            },
                        }
                    },
                    Err(err) => {
                        for e in err.messages {
                            eprintln!("Compiler Error:\n");
                            eprintln!("\t{}", line);
                            let start = e.span.start;
                            let spaces: String = std::iter::repeat(' ')
                                .take(start)
                                .collect();

                            let up_arrow: String = std::iter::repeat('^')
                                .take(e.span.width())
                                .collect();

                            eprintln!("\t{}{} {}", spaces, up_arrow, e.message.to_lowercase());
                        }
                    },
                }
                rl.save_history(".lambda").unwrap();
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
    rl.save_history(".lambda").unwrap();
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
