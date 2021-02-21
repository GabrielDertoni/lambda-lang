pub mod ast;
pub mod tokens;
pub mod error;
pub mod span;


use std::cell::Cell;

use ast::*;
use error::*;
use span::*;

pub type Result<T> = std::result::Result<T, Error>;

pub struct ParseStream<'a> {
    pub scope: Span,
    curr_span: Cell<Span>,
    original: &'a str,
    remaining: Cell<&'a str>,
}

impl<'a> ParseStream<'a> {
    pub fn new(scope: Span, s: &'a str) -> ParseStream<'a> {
        ParseStream {
            scope,
            curr_span: Cell::new(scope),
            original: s,
            remaining: Cell::new(s),
        }
    }

    fn advance(&self) {
        let mut span = self.curr_span();

        let mut it = self.remaining.get().chars();
        if it.next().is_some() {
            span.start += 1;
        }
        self.remaining.set(it.as_str());

        self.curr_span.set(span);
    }

    fn advance_by(&self, n: usize) {
        let mut span = self.curr_span();

        let mut it = self.remaining.get().chars();
        for _ in 0..n {
            if it.next().is_some() {
                span.start += 1;
            }
        }
        self.remaining.set(it.as_str());

        self.curr_span.set(span);
    }

    fn get(&'a self) -> Option<char> {
        self.remaining.get().chars().nth(0)
    }

    fn peek(&'a self, n: usize) -> Option<&'a str> {
        self.remaining.get().get(..n)
    }

    fn parse<T: Parser>(&'a self) -> Result<T> {
        T::parse(self)
    }

    fn is_empty(&self) -> bool {
        self.remaining.get().len() == 0
    }

    fn curr_span(&self) -> Span {
        self.curr_span.get()
    }
}

impl<'a> From<&'a str> for ParseStream<'a> {
    fn from(s: &'a str) -> Self {
        ParseStream::new(Span::from(s), s)
    }
}


pub trait Parser: Sized {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Self>;
}

impl<T: Parser> Parser for Box<T> {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Box<T>> {
        Ok(Box::new(input.parse()?))
    }
}


impl Parser for Program {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Program> {
        let mut stmts = Vec::new();

        while !input.is_empty() {
            stmts.push(input.parse()?);
        }

        Ok(Program { stmts })
    }
}

impl Parser for Stmt {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Stmt> {
        input.parse()
            .map(|macro_def| Stmt::Macro(macro_def))
            .or_else(|_| Ok(Stmt::Expr(input.parse()?)))
    }
}

impl Parser for Macro {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Macro> {
        Ok(Macro {
            name: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parser for Expr {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Expr> {
        let span = input.curr_span();
        let next = input
            .peek(1)
            .ok_or(Error::new(span.start(), "Unexpected end of input"))?;

        let peek_stream = ParseStream::new(Span::new(span.start, span.start + 1), next);
        if tokens::Lambda::parse(&peek_stream).is_ok() {
            Ok(Expr::Lambda(input.parse()?))
        } else if tokens::Quote::parse(&peek_stream).is_ok() {
            Ok(Expr::Literal(input.parse()?))
        } else {
            let rhs = input.parse()?;
            if input.is_empty() {
                Ok(Expr::Close(rhs))
            } else {
                let lhs = input.parse()?;
                Ok(Expr::Appl(Appl { rhs, lhs }))
            }
        }
    }
}

impl Parser for Lambda {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Lambda> {
        Ok(Lambda {
            lambda_token: input.parse()?,
            var: input.parse()?,
            dot_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl Parser for Appl {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Appl> {
        Ok(Appl {
            rhs: input.parse()?,
            lhs: input.parse()?,
        })
    }
}

impl Parser for Close {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Self> {
        if tokens::LParen::parse(input).is_ok() {
            let close = Close::Paren(input.parse()?);
            tokens::RParen::parse(input)?;
            Ok(close)
        } else {
            Ok(Close::Var(input.parse()?))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_literal_parser() {
        let input = "\\a. a a";
        let stream = ParseStream::from(input);
        match Expr::parse(&stream) {
            Ok(res) => eprintln!("{:#?}", res),
            Err(e)  => eprintln!("{}", e),
        }
    }
}
