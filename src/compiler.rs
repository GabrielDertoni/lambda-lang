use std::collections::VecDeque;
use std::collections::HashMap;

use crate::parser;
use crate::parser::{ Result, Parser };
use crate::parser::ast;
use crate::parser::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'a> {
    Lambda {
        param: usize,
        expr: Box<Expr<'a>>,
    },
    Appl {
        f: Box<Expr<'a>>,
        arg: Box<Expr<'a>>,
    },
    Var(usize),
    Literal(&'a str),
}

pub fn compile_program<'lit>(s: &str, heap: &'lit mut Vec<String>) -> Result<Expr<'lit>> {
    let stream = parser::ParseStream::from(s);
    let ast = ast::Program::parse(&stream)?;

    let literals = alloc_prog_literals(&ast);
    literals.into_iter().for_each(|el| heap.push(el));

    // TODO: Allow loop macros like A refer to B and B refer to A
    let mut macros = HashMap::new();
    for (i, stmt) in ast.stmts.iter().enumerate() {
        let mut compiler = Compiler::new(heap, &macros);
        match stmt {
            ast::Stmt::Macro(mac) => {
                let compiled = compiler.compile_expr(&mac.value)?;
                macros.insert(&mac.name.name, compiled);
            },
            ast::Stmt::Expr(expr) => {
                assert!(i == ast.stmts.len() - 1);
                return Ok(compiler.compile_expr(expr)?);
            }
        }
    }

    Err(Error::new(stream.scope, "Expected an expression"))
}

pub fn alloc_prog_literals(prog: &ast::Program) -> Vec<String> {
    let mut literals = Vec::new();
    let mut expr_queue = VecDeque::new();

    for stmt in prog.stmts.iter() {
        let expr = match stmt {
            ast::Stmt::Macro(mac) => &mac.value,
            ast::Stmt::Expr(expr) => &expr,
        };
        expr_queue.push_back(expr);

        while let Some(expr) = expr_queue.pop_front() {
            match expr {
                ast::Expr::Lambda(lambda) => expr_queue.push_back(lambda.expr.as_ref()),
                ast::Expr::Close(close)   => alloc_close_literals(&close, &mut literals, &mut expr_queue),
                ast::Expr::Appl(appl)     => {
                    alloc_close_literals(&appl.lhs, &mut literals, &mut expr_queue);
                    alloc_close_literals(&appl.rhs, &mut literals, &mut expr_queue);
                },
            };
        }
    }

    literals
}

#[inline]
fn alloc_close_literals<'a>(close: &'a ast::Close, literals: &mut Vec<String>, expr_queue: &mut VecDeque<&'a ast::Expr>) {
    match close {
        ast::Close::Paren(e)     => expr_queue.push_back(e.as_ref()),
        ast::Close::Literal(lit) => literals.push(lit.content.clone()),
        ast::Close::Var(_)       => (),
    }
}

/*
 * The 'expr lifetime is used for references that point into some ast::Expr, this
 * may be a pointer to the string in a Literal token, for example.
 * 'mac is used to referer to the reference to the macros hashmap.
 * 'lit is used to refer to the reference to the vector that contains all literal
 * strings in the program. 'lit must outlive 'expr and 'mac because 'mac points
 * into 'lit and 'expr will usualy be used to create get a refenrece into 'lit.
 */
struct Compiler<'expr, 'mac, 'lit: 'expr + 'mac> {
    literals: &'lit Vec<String>,
    macros: &'mac HashMap<&'expr str, Expr<'lit>>,
    var_name_to_id: HashMap<&'expr str, usize>,
}

struct Test<'a> {
    reference: &'a HashMap<&'a str, usize>,
}

impl<'expr, 'mac, 'lit: 'expr + 'mac> Compiler<'expr, 'mac, 'lit> {
    fn new(literals: &'lit Vec<String>, macros: &'mac HashMap<&'expr str, Expr<'lit>>) -> Compiler<'expr, 'mac, 'lit> {
        Compiler { literals, var_name_to_id: HashMap::new(), macros }
    }

    fn compile_expr(&mut self, expr: &'expr ast::Expr) -> Result<Expr<'lit>> {
        let mut new_var = None;
        let compiled = match expr {
            ast::Expr::Lambda(lambda) => {
                let param = self.var_name_to_id.len();
                assert!(self.var_name_to_id.insert(&lambda.var.name, param).is_none());
                new_var = Some(&lambda.var.name);

                Expr::Lambda {
                    param,
                    expr: self.compile_expr(&lambda.expr)?.into()
                }
            },
            ast::Expr::Close(close) => self.compile_close(&close)?,
            ast::Expr::Appl(appl)   => {
                Expr::Appl {
                    f:   self.compile_close(&appl.rhs)?.into(),
                    arg: self.compile_close(&appl.lhs)?.into(),
                }
            },
        };

        // If a variable has been added to scope, remove it here, where the scope is no more.
        if let Some(name) = new_var {
            self.var_name_to_id.remove(&name.as_ref());
        }

        Ok(compiled)
    }

    fn compile_close(&mut self, close: &'expr ast::Close) -> Result<Expr<'lit>> {
        Ok(match close {
            ast::Close::Paren(expr) => self.compile_expr(expr.as_ref())?,
            ast::Close::Var(var)    => {
                let var_id = self.var_name_to_id
                    .get(&var.name.as_str())
                    .ok_or(Error::new(var.span, "Use of undeclared variable"))?
                    .to_owned();

                Expr::Var(var_id)
            },
            ast::Close::Literal(lit) => {
                let s = self.literals
                    .iter()
                    .find(|el| *el == &lit.content)
                    .unwrap();

                Expr::Literal(s)
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compilation() {
        let input = "(\\a. a a) (\\a. a a)";
        let mut heap = Vec::new();
        let result = compile_program(input, &mut heap).unwrap();
        println!("{:#?}", result);
        // let literals = 
    }
}
