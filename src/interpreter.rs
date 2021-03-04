use std::collections::{ HashSet, HashMap };
use std::rc::Rc;
use std::ptr::NonNull;

use crate::error::RuntimeError;

const MAX_EVAL_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    // TODO: Allow for a parameter list.
    Lambda {
        param: usize,
        expr: Box<Expr>,
    },
    // TODO: Make this into a vec of expression
    Appl {
        f: Box<Expr>,
        arg: Box<Expr>,
    },
    MacroRef(Rc<Macro>),
    Var(usize),
    Literal(Rc<String>),
    Nothing,
}

pub struct Macro {
    pub expr: Expr,
    name: NonNull<str>,
}

impl Macro {
    pub fn new(expr: Expr, name: NonNull<str>) -> Macro {
        Macro { expr, name }
    }

    unsafe fn get_name(&self) -> &str {
        self.name.as_ref()
    }
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

pub struct Executable {
    pub expr: Expr,
    // No entries should be removed from this hashmap.
    pub macros: HashMap<String, Rc<Macro>>,
    pub literals: HashSet<Rc<String>>,
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


use std::borrow::Cow;
impl Expr {
    /// Verifies if an expression is in Weak Head Normal Form.
    /// An expression is in WHNF if all of the "left hand side" things are
    /// evaluated.
    pub fn is_whnf(&self) -> bool {
        let mut next = Some(self);
        while let Some(curr) = next.take() {
            match curr {
                Expr::Nothing    |
                Expr::Literal(_) |
                Expr::Var(_)           => return true,
                Expr::Appl { f, .. }   => next = Some(f),
                Expr::MacroRef(mac)    => next = Some(&mac.as_ref().expr),
                lamb@Expr::Lambda {..} 
                    if lamb.is_n_reducible() => return false,
                _                            => return true,
            }
        }
        return false;
    }

    /// Verifies if an expression is n-reducible.
    /// An expression is n-reducible in case it is an expression like \a. f a.
    /// In this case, the n-reduction would be \a. f a -> f, meaning that the
    /// initial expression was n-reducible.
    pub fn is_n_reducible(&self) -> bool {
        match self {
            Expr::Lambda {
                param,
                expr: box Expr::Appl {
                    arg: box Expr::Var(arg_var),
                    ..
                },
            } if param == arg_var => true,
            _                     => false,
        }
    }

    pub fn is_normal_form(&self) -> bool {
        match self {
            Expr::Nothing    |
            Expr::Literal(_) |
            Expr::Var(_)                 => true,
            Expr::Appl { f, arg }        => f.is_normal_form() && arg.is_normal_form(),
            Expr::MacroRef(mac)          => mac.as_ref().expr.is_normal_form(),
            lamb@Expr::Lambda {..}
                if lamb.is_n_reducible() => false,
            Expr::Lambda{..}             => true,
        }
    }

    /// Converts all used variables so that the ones in the top of the tree will
    /// start at 0 and increase in valua as they go down the expression tree.
    /// This is necessary in order to compare two different expressions.
    pub fn alpha_convert(&mut self) {
        self.alpha_convert_from(0);
    }

    pub fn alpha_convert_from(&mut self, start: usize) {
        // Cow is used so the vec is not cloned unless it is really needed.
        let conversion_table = Cow::Owned(Vec::new());
        self.alpha_convert_with_table(conversion_table, start);
    }

    fn alpha_convert_with_table(&mut self, mut conversion_table: Cow<Vec<usize>>, start: usize) {
        match self {
            Expr::Literal(_)  |
            Expr::MacroRef(_) | // Macros are already always alpha simplified.
            Expr::Nothing         => (),
            Expr::Appl { f, arg } => {
                // This clone is necessary because we can't let the local
                // variables that may be defined in expression `f` to be used
                // in the `arg` expression.
                f.alpha_convert_with_table(Cow::Borrowed(conversion_table.as_ref()), start);
                arg.alpha_convert_with_table(conversion_table, start);
            },
            Expr::Lambda { param, expr } => {
                // This is ok because a new parameter found in the tree will
                // always have a larger value then its predecessors, so the
                // `conversion_table` will remain sorted.
                conversion_table.to_mut().push(*param);
                *param = conversion_table.len() - 1 + start;
                expr.alpha_convert_with_table(conversion_table, start);
            },
            Expr::Var(v) => {
                assert!(conversion_table.is_sorted());
                let pos = conversion_table
                    .binary_search(v)
                    .expect("Found var not present in conversion table.");

                *v = pos + start;
            },
        }
    }

    pub fn get_biggest_var_id(&self) -> Option<usize> {
        match self {
            Expr::Nothing    |
            Expr::Literal(_)            => None,
            Expr::MacroRef(mac)         => mac.as_ref().expr.get_biggest_var_id(),
            Expr::Appl { f, arg }       => {
                f.get_biggest_var_id()
                    .map(|v| {
                        arg.get_biggest_var_id()
                            .map_or(v, |v2| std::cmp::max(v, v2))
                    })
            },
            Expr::Lambda { param, expr } => {
                expr.get_biggest_var_id()
                    .map(|v| std::cmp::max(*param, v))
                    .or(Some(*param))
            },
            Expr::Var(v)                => Some(*v),
        }
    }

    // Perform beta-reduction all the way to normal form.
    pub fn eval(&mut self) -> Result<&mut Expr, RuntimeError> {
        self.eval_depth(0, false)
    }

    pub fn eval_depth(&mut self, depth: usize, eval_macros: bool) -> Result<&mut Expr, RuntimeError> {
        if depth > MAX_EVAL_DEPTH {
            return Err(RuntimeError::RecursionDepthExceeded);
        }

        for _ in 0..MAX_EVAL_DEPTH {
            match self {
                Expr::Literal(_) |
                Expr::Var(_)        => return Ok(self),
                Expr::Lambda { .. } => {
                    /*
                    // NOTE: Maybe this should happen...
                    expr.eval_depth(depth + 1, false)?;
                    */
                    return Ok(self);
                },
                Expr::Appl { f, .. }      => {
                    f.eval_depth(depth + 1, true)?;
                    let biggest_f_var_id = f.get_biggest_var_id().unwrap_or(0);
                    let owned = self.take();
                    if let Expr::Appl {
                        f: box Expr::Lambda {
                            param,
                            box mut expr
                        },
                        mut arg
                    } = owned {
                        arg.alpha_convert_from(biggest_f_var_id + 1);
                        expr.subst(param, *arg);
                        expr.alpha_convert();
                        drop(self.replace(expr));
                    } else {
                        drop(self.replace(owned));
                        return Ok(self);
                    }
                },
                Expr::MacroRef(ptr) => {
                    if !ptr.as_ref().expr.is_normal_form() || eval_macros {
                        let expr = ptr.expr.clone();
                        // expr.eval_depth(depth, eval_macros)?;
                        drop(std::mem::replace(self, expr));
                    } else {
                        return Ok(self);
                    }
                },
                Expr::Nothing => {
                    return Err(RuntimeError::NothingEval);
                }
            }
        }
        // self.eval_depth(depth + 1)?;
        // Ok(self)
        return Err(RuntimeError::IterationExceeded);
    }

    fn subst(&mut self, var: usize, new_expr: Expr) {
        match self {
            Expr::Lambda { expr, .. } => expr.subst(var, new_expr),
            Expr::Appl { f, arg }     => {
                f.subst(var, new_expr.clone());
                arg.subst(var, new_expr);
            },
            Expr::MacroRef(ptr)  => {
                let mut expr = ptr.expr.clone();
                expr.subst(var, new_expr);
            }
            Expr::Var(v)         => if *v == var { *self = new_expr },
            Expr::Literal(_) |
            Expr::Nothing        => (),
        }
    }
}

// Pure implementations (no mutation, lots of cloning)
impl Expr {
    pub fn pure_alpha_convert(&self) -> Expr {
        let mut clone = self.clone();
        clone.alpha_convert();
        clone
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lambda { param, expr } => write!(f, "λ{}. {}", Expr::Var(*param), expr),
            Expr::Appl {
                f: box Expr::Literal(a),
                arg: box Expr::Literal(b),
            } => write!(f, "\"{}{}\"", a.as_ref(), b.as_ref()),
            
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
            Expr::Literal(s)        => write!(f, "{}", s),
            Expr::MacroRef(ptr)     => {
                let name = unsafe { &ptr.as_ref().name.as_ref() };
                write!(f, "{}", name)
            }
            Expr::Nothing           => write!(f, "[nothing expression]"),
        }
    }
}

impl std::default::Default for Expr {
    fn default() -> Expr {
        Expr::Nothing
    }
}

impl Expr {
    pub fn take(&mut self) -> Expr {
        std::mem::take(self)
    }

    pub fn replace(&mut self, expr: Expr) -> Expr {
        std::mem::replace(self, expr)
    }
}
