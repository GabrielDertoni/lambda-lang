use std::collections::{ HashMap, HashSet, VecDeque };
use std::rc::Rc;

use crate::parser;
use crate::parser::{ Result, Parser };
use crate::parser::ast;
use crate::parser::error::Error;
use crate::interpreter::{ Expr, Executable, Macro };

pub fn compile_program(s: &str) -> Result<Executable> {
    let stream = parser::ParseStream::from(s);
    let ast = ast::Program::parse(&stream)?;

    let literals = alloc_prog_literals(&ast);

    // TODO: Allow loop macros like A refer to B and B refer to A
    let mut macros = HashMap::new();
    for (i, stmt) in ast.stmts.iter().enumerate() {
        let mut compiler = Compiler::new(&literals, &macros);
        match stmt {
            ast::Stmt::Macro(mac) => {
                let compiled = compiler.compile_expr(&mac.value)?;
                let name = mac.name.name.clone();
                let name_ptr = std::ptr::NonNull::from(name.as_ref());
                macros.insert(name, Rc::new(Macro::new(compiled, name_ptr)));
            },
            ast::Stmt::Expr(expr) => {
                assert!(i == ast.stmts.len() - 1);
                let compiled = compiler.compile_expr(expr)?;
                return Ok(Executable::new(compiled, macros, literals));
            }
        }
    }

    Err(Error::new(stream.scope, "Expected an expression"))
}

pub enum StmtReturn {
    Macro(String),
    Expr(Expr),
}

pub fn compile_stmt(
    s: &str,
    literals: &mut HashSet<Rc<String>>,
    macros: &mut HashMap<String, Rc<Macro>>
) -> Result<StmtReturn>
{
    let stream = parser::ParseStream::from(s);
    let stmt = ast::Stmt::parse(&stream)?;

    alloc_stmt_literals(&stmt, literals);

    let mut compiler = Compiler::new(literals, &macros);
    match stmt {
        ast::Stmt::Macro(mac) => {
            let compiled = compiler.compile_expr(&mac.value)?;
            let name = mac.name.name.clone();
            let name_ptr = std::ptr::NonNull::from(name.as_ref());
            macros.insert(name, Rc::new(Macro::new(compiled, name_ptr)));
            Ok(StmtReturn::Macro(mac.name.name.to_owned()))
        },
        ast::Stmt::Expr(expr) => {
            Ok(StmtReturn::Expr(compiler.compile_expr(&expr)?))
        }
    }
}

// Traverses the entire AST and finds all string literals in the program and
// copies them into a vector.
pub fn alloc_prog_literals(prog: &ast::Program) -> HashSet<Rc<String>> {
    let mut literals = HashSet::new();

    for stmt in prog.stmts.iter() {
        alloc_stmt_literals(stmt, &mut literals);
    }

    literals
}

fn alloc_stmt_literals(stmt: &ast::Stmt, literals: &mut HashSet<Rc<String>>) {
    let mut expr_queue = VecDeque::new();

    let expr = match stmt {
        ast::Stmt::Macro(mac) => &mac.value,
        ast::Stmt::Expr(expr) => &expr,
    };
    expr_queue.push_back(expr);

    while let Some(expr) = expr_queue.pop_front() {
        match expr {
            ast::Expr::Lambda(lambda) => expr_queue.push_back(lambda.expr.as_ref()),
            ast::Expr::Close(close)   => alloc_close_literals(&close, literals, &mut expr_queue),
            ast::Expr::Appl(appl)     => {
                alloc_close_literals(&appl.lhs, literals, &mut expr_queue);
                alloc_close_literals(&appl.rhs, literals, &mut expr_queue);
            },
        }
    }
}

#[inline]
fn alloc_close_literals<'a>(
    close: &'a ast::Close,
    literals: &mut HashSet<Rc<String>>,
    expr_queue: &mut VecDeque<&'a ast::Expr>
) {
    match close {
        ast::Close::Paren(e)     => expr_queue.push_back(e.as_ref()),
        ast::Close::Literal(lit) => {
            literals.insert(Rc::new(lit.content.clone()));
        },
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
struct Compiler<'expr, 'lit> {
    literals: &'lit HashSet<Rc<String>>,
    macros: &'lit HashMap<String, Rc<Macro>>,
    var_name_to_id: HashMap<&'expr str, usize>,
}

impl<'expr, 'lit> Compiler<'expr, 'lit> {
    fn new(literals: &'lit HashSet<Rc<String>>, macros: &'lit HashMap<String, Rc<Macro>>) -> Compiler<'expr, 'lit> {
        Compiler { literals, macros, var_name_to_id: HashMap::new() }
    }

    fn compile_expr(&mut self, expr: &'expr ast::Expr) -> Result<Expr> {
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
                    f:   self.compile_close(&appl.lhs)?.into(),
                    arg: self.compile_close(&appl.rhs)?.into(),
                }
            },
        };

        // If a variable has been added to scope, remove it here, where the scope is no more.
        if let Some(name) = new_var {
            self.var_name_to_id.remove(&name.as_ref());
        }

        Ok(compiled)
    }

    fn compile_close(&mut self, close: &'expr ast::Close) -> Result<Expr> {
        Ok(match close {
            ast::Close::Paren(expr) => self.compile_expr(expr.as_ref())?,
            ast::Close::Var(var)    => {
                match self.var_name_to_id.get(&var.name.as_str()) {
                    Some(&var_id) => Expr::Var(var_id),
                    None          => {
                        let mac = self.macros
                            .get(&var.name)
                            .ok_or_else(|| Error::new(var.span, "Use of undeclared variable or macro"))?;

                        Expr::MacroRef(Rc::clone(mac))
                    },
                }
            },
            ast::Close::Literal(lit) => {
                let s = self.literals.get(&lit.content).unwrap();
                Expr::Literal(Rc::clone(s))
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
        assert!(compile_program(input).is_ok());
    }
}
