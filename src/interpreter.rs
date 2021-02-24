
use std::collections::{ HashSet, HashMap };
use std::rc::Rc;
use std::ptr::NonNull;

const MAX_EVAL_DEPTH: usize = 64;

pub struct Macro {
    pub expr: Expr,
    name: NonNull<str>,
}

impl PartialEq for Macro {
    fn eq(&self, other: &Self) -> bool {
        self.expr == other.expr
    }
}

impl std::fmt::Debug for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { self.name.as_ref() };
        write!(f, "def {} = {:?}", name, self.expr)
    }
}

impl Eq for Macro {}

impl Macro {
    pub fn new(expr: Expr, name: NonNull<str>) -> Macro {
        Macro { expr, name }
    }
}

pub struct Executable {
    pub expr: Expr,
    // No entries should be removed from this hashmap.
    pub macros: HashMap<String, Rc<Macro>>,
    pub literals: HashSet<Rc<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Lambda {
        param: usize,
        expr: Box<Expr>,
    },
    Appl {
        f: Box<Expr>,
        arg: Box<Expr>,
    },
    MacroRef(Rc<Macro>),
    Var(usize),
    Literal(Rc<String>),
}

#[derive(Debug)]
pub enum RuntimeError {
    Unknown,
    RecursionDepthExceeded,
}

impl RuntimeError {
    pub fn new() -> RuntimeError {
        RuntimeError::Unknown
    }
}

use std::fmt::{ Display, Formatter };
impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::Unknown                => write!(f, "Unknown error"),
            RuntimeError::RecursionDepthExceeded => write!(f, "Recursion depth exceeded"),
        }
    }
}

impl Executable {
    pub fn new(expr: Expr, macros: HashMap<String, Rc<Macro>>, literals: HashSet<Rc<String>>) -> Executable {
        Executable { expr, macros, literals }
    }

    pub fn eval(&mut self) -> Result<&mut Executable, RuntimeError> {
        match self.expr.eval() {
            Ok(_)  => Ok(self),
            Err(e) => Err(e),
        }
    }
}

fn memapply<T, F: FnOnce(T) -> T>(dest: &mut T, f: F) {
    // This is ok because the return value of `f` is a valid value of type T.
    // This means that, even though for a moment `dest` receives an unsafe
    // value, right away it gets replaced by a valid one. It also forgets the
    // zeroed value so no destructor is called.
    unsafe {
        let zeroed = std::mem::zeroed();
        let owned = std::mem::replace(dest, zeroed);
        std::mem::forget(std::mem::replace(dest, f(owned)));
    }
}

impl Expr {
    // Perform beta-reduction.
    pub fn eval(&mut self) -> Result<&mut Expr, RuntimeError> {
        self.eval_depth(0)
    }

    pub fn eval_depth(&mut self, depth: usize) -> Result<&mut Expr, RuntimeError> {
        if depth > MAX_EVAL_DEPTH {
            return Err(RuntimeError::RecursionDepthExceeded);
        }

        match self {
            Expr::Literal(_) |
            Expr::Var(_)              => return Ok(self),
            Expr::Lambda { expr, .. } => {
                expr.eval_depth(depth + 1)?;
                return Ok(self);
            },
            Expr::Appl { f, .. }      => {
                f.eval_depth(depth + 1)?;
                memapply(self, |owned| match owned {
                    Expr::Appl {
                        f: box Expr::Lambda {
                            param,
                            mut expr
                        },
                        arg
                    } => {
                        expr.subst(param, *arg);
                        *expr
                    },
                    _ => owned,
                });
            },
            Expr::MacroRef(ptr) => {
                let mut expr = ptr.expr.clone();
                // Q: Should this type of evaluation increase the evaluation depth?
                expr.eval_depth(depth + 1)?;
                drop(std::mem::replace(self, expr));
            }
        }
        self.eval_depth(depth + 1)?;
        Ok(self)
    }

    fn subst(&mut self, var: usize, new_expr: Expr) {
        match self {
            Expr::Lambda { expr, .. } => expr.subst(var, new_expr),
            Expr::Appl { f, arg }     => {
                f.subst(var, new_expr.clone());
                arg.subst(var, new_expr);
            },
            Expr::Var(v)         => if *v == var { *self = new_expr },
            Expr::Literal(_)     => (),
            Expr::MacroRef(ptr)  => {
                let mut expr = ptr.expr.clone();
                expr.subst(var, new_expr);
            }
        }
    }
}

impl<'a> std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lambda { param, expr } => write!(f, "{}. {}", Expr::Var(*param), expr),
            Expr::Appl { f: func, arg }  => {
                match func.as_ref() {
                    Expr::Lambda { .. }  => write!(f, "({})", func),
                    _ => write!(f, "{}", func),
                }?;
                write!(f, " ")?;
                match arg.as_ref() {
                    Expr::Lambda { .. } | Expr::Appl{ .. } => write!(f, "({})", arg),
                    _ => write!(f, "{}", arg),
                }
            },
            Expr::Var(v)            => write!(f, "{}", (*v as u8 + 97) as char),
            Expr::Literal(s)        => write!(f, "\"{}\"", s),
            Expr::MacroRef(ptr)     => {
                let name = unsafe { &ptr.as_ref().name.as_ref() };
                write!(f, "{}", name)
            }
        }
    }
}
