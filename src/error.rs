use std::fmt;

pub trait Error: fmt::Display {

}

#[derive(Debug)]
pub enum RuntimeError {
    Unknown,
    NothingEval,
    RecursionDepthExceeded,
    IterationExceeded,
}

impl RuntimeError {
    #[inline]
    pub fn new() -> RuntimeError {
        RuntimeError::default()
    }
}

impl std::default::Default for RuntimeError {
    #[inline]
    fn default() -> RuntimeError {
        RuntimeError::Unknown
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Unknown                => {
                write!(f, "Unknown error")?;
            },
            RuntimeError::NothingEval            => {
                write!(f, "Tried to evaluate a nothing expression")?;
            },
            RuntimeError::RecursionDepthExceeded => {
                writeln!(f, "Recursion depth exceeded:")?;
                write!(f, "\t It is impossible to find a Weak Head Normal Form.")?;
            },
            RuntimeError::IterationExceeded      => {
                writeln!(f, "Max eval iterations exceeded:")?;
                write!(f, "\tIt is possible to find a Weak Head Normal Form, but not a Normal Form.")?;
            },
        }
        Ok(())
    }
}

impl Error for RuntimeError {}
