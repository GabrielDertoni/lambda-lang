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

    pub fn skip_whitespace(&'a self) {
        while let Some(c) = self.get() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn advance(&self) {
        let mut span = self.curr_span();

        let mut it = self.remaining.get().chars();
        if it.next().is_some() {
            span.start += 1;
        }
        self.remaining.set(it.as_str());

        self.curr_span.set(span);
    }

    pub fn advance_by(&self, n: usize) {
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

    #[inline]
    pub fn get(&'a self) -> Option<char> {
        self.remaining.get().chars().nth(0)
    }
    
    #[inline]
    pub fn get_remaining(&'a self) -> &str {
        self.remaining.get()
    }

    #[inline]
    pub fn peek(&'a self, n: usize) -> Option<&'a str> {
        self.remaining.get().get(..n)
    }

    #[inline]
    pub fn parse<T: Parser>(&'a self) -> Result<T> {
        T::parse(self)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.remaining.get().len() == 0
    }

    #[inline]
    pub fn curr_span(&self) -> Span {
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
        input.skip_whitespace();
        let next = input
            .peek(1)
            .ok_or(Error::new(span.start(), "Unexpected end of input"))?;

        let peek_stream = ParseStream::new(Span::new(span.start, span.start + 1), next);
        if tokens::Lambda::parse(&peek_stream).is_ok() {
            let lamb = match input.parse() {
                Ok(lamb) => lamb,
                Err(err) => { println!("{}", err); return Err(err); },
            };
            Ok(Expr::Lambda(lamb))
        } else {
            let rhs = input.parse()?;
            input.skip_whitespace();
            if input.is_empty() || input.peek(1).map(|s| s == ")").unwrap_or(false) {
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
    /*
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Close> {
        let lookahead = input.peek(1)
            .ok_or_else(|| Error::new(input.curr_span().start(), "Unexpected end of input"))?;

        if lookahead == "(" {
            let (content, _paren) = tokens::parse_parenthesis(input)?;
            // let expr = content.parse()?;
            let expr = Box::new(Expr::parse(&content)?);
            Ok(Close::Paren(expr))
        } else {
            Ok(Close::Var(input.parse()?))
        }
    }
    */
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Close> {
        input.skip_whitespace();
        let lookahead = input.peek(1)
            .ok_or_else(|| Error::new(input.curr_span().start(), "Unexpected end of input"))?;

        if tokens::LParen::parse(input).is_ok() {
            let expr = input.parse()?;
            tokens::RParen::parse(input)?;
            Ok(Close::Paren(expr))
        } else if lookahead == "\"" {
            Ok(Close::Literal(input.parse()?))
        } else {
            Ok(Close::Var(input.parse()?))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_stmt() {
        let stream = ParseStream::from("\\a. a a");
        assert!(Stmt::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.remaining.get());
    }

    #[test]
    fn test_literal_parser() {
        let stream = ParseStream::from("\\a. a a");
        assert!(Expr::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.remaining.get());
    }

    #[test]
    fn test_paren() {
        let stream = ParseStream::from("(\\a. a a)");
        assert!(Expr::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.remaining.get());
    }

    #[test]
    fn test_var() {
        let stream = ParseStream::from("a");
        assert!(tokens::Var::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.remaining.get());
    }

    #[test]
    fn test_literal() {
        let stream = ParseStream::from("\"hello world\"");
        assert!(tokens::Literal::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.remaining.get());
    }
}
