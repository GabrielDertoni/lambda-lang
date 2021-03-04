use std::iter::Extend;
use std::fmt;

use crate::span::Span;

#[derive(Clone)]
pub struct Error {
    pub messages: Vec<ErrorMessage>,
}

impl Error {
    pub fn new<T: ToString>(span: Span, val: T) -> Error {
        Error { messages: vec![ErrorMessage::new(span, val)] }
    }

    pub fn new_compiler_err<T: ToString>(val: T) -> Error {
        panic!("CompilerError: {}", val.to_string())
    }

    pub fn cover_span(&self) -> Span {
        self.messages.iter()
            .map(|msg| msg.span)
            .fold_first(|a, b| a.merge(b))
            .unwrap()
    }

    pub fn or(self, other: Error) -> Error {
        if self.cover_span().start > other.cover_span().start {
            self
        } else {
            other
        }
    }

    pub fn push<T: ToString>(&mut self, span: Span, val: T) {
        self.messages.push(ErrorMessage::new(span, val.to_string()));
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Errors:\n")?;
        for msg in self.messages.iter() {
            write!(f, "\t{}", msg)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Extend<ErrorMessage> for Error {
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = ErrorMessage>,
    {
        self.messages.extend(iter)
    }
}

#[derive(Clone)]
pub struct ErrorMessage {
    pub span: Span,
    pub message: String,
}

impl ErrorMessage {
    fn new<T: ToString>(span: Span, val: T) -> ErrorMessage {
        ErrorMessage { span, message: val.to_string() }
    }
}

impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at bytes {} to {}", self.message, self.span.start, self.span.end)
    }
}
