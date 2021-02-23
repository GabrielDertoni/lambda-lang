use crate::compiler::Expr;

const MAX_EVAL_DEPTH: usize = 64;

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
    fn subst(&mut self, var: usize, new_expr: Expr) {
        match self {
            Expr::Lambda { expr, .. } => expr.subst(var, new_expr),
            Expr::Appl { f, arg }     => {
                f.subst(var, new_expr.clone());
                arg.subst(var, new_expr);
            },
            Expr::Var(v)         => if *v == var { *self = new_expr },
            Expr::Literal(_)     => (),
        }
    }

    // Perform beta-reduction.
    pub fn eval(&mut self) -> Result<&'_ mut Expr, String> {
        let mut eval_stack: Vec<&mut Expr> = Vec::new();
        let mut curr: &mut Expr = self;
        let mut i = 0;

        'eval: loop {
            let curr_ptr: *mut _ = curr;
            curr = match curr {
                Expr::Appl { f, .. } => {
                    match f.as_mut() {
                        Expr::Lambda {..} => unsafe {
                            memapply(&mut *curr_ptr, |owned| {
                                let (f, arg) = owned.unwrap_appl();
                                let (param, mut expr) = f.unwrap_lambda();
                                expr.subst(param, *arg);
                                return *expr;
                            });

                            &mut *curr_ptr
                        },
                        expr => unsafe {
                            eval_stack.push(&mut *curr_ptr);
                            expr
                        },
                    }
                },
                _ => {
                    match eval_stack.pop() {
                        None => break 'eval Ok(self),
                        Some(v) => v,
                    }
                },
            };

            if i < MAX_EVAL_DEPTH && eval_stack.len() < MAX_EVAL_DEPTH {
                i += 1;
            } else {
                break 'eval Err(format!("Maximum recursion depth excedeed at {}", self))
            }
        }
    }

    #[inline]
    fn unwrap_appl(self) -> (Box<Expr>, Box<Expr>) {
        if let Expr::Appl { f, arg } = self {
            (f, arg)
        } else {
            unreachable!()
        }
    }

    #[inline]
    fn unwrap_lambda(self) -> (usize, Box<Expr>) {
        if let Expr::Lambda { param, expr } = self {
            (param, expr)
        } else {
            unreachable!()
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
        }
    }
}

pub fn print_expr(expr: &Expr, mem_static: &Vec<String>) {
    match expr {
        Expr::Lambda { param, expr } => println!("{}. {}", Expr::Var(*param), expr),
        Expr::Appl { f: func, arg }  => {
            match func.as_ref() {
                Expr::Lambda { .. } => println!("({})", func),
                _                   => println!("{}", func),
            };

            println!(" ");

            match arg.as_ref() {
                Expr::Lambda { .. } |
                Expr::Appl { .. }    => println!("({})", arg),
                _                    => println!("{}", arg),
            }
        },
        Expr::Var(v)     => println!("{}", (*v as u8 + 97) as char),
        Expr::Literal(i) => println!("\"{}\"", mem_static[*i]),
    }
}
