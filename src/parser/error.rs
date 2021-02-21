use super::span::Span;

pub struct Error {
    messages: Vec<ErrorMessage>,
}

impl Error {
    pub fn new<T: ToString>(span: Span, val: T) -> Error {
        Error { messages: vec![ErrorMessage::new(span, val)] }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Errors:\n")?;
        for msg in self.messages.iter() {
            write!(f, "\t{}", msg)?;
        }
        Ok(())
    }
}

struct ErrorMessage {
    span: Span,
    message: String,
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
