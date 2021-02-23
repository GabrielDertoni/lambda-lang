pub mod ast;
pub mod tokens;
pub mod error;
pub mod span;


use std::cell::Cell;

use ast::*;
use error::*;
use span::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
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

    pub fn skip_whitespace(&self) {
        while let Some(c) = self.get() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    // Advances the stream until the next valid token, that means that it will
    // automatically get rid of any and all whitespaces.
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

    // Advances the stream, skipping any space, and returns the next
    // non-whitespace char.
    pub fn next(&self) -> Option<char> {
        self.skip_whitespace();
        self.get()
    }

    pub fn fork(&self) -> ParseStream<'a> {
        self.clone()
    }

    #[inline]
    pub fn get(&self) -> Option<char> {
        self.remaining.get().chars().nth(0)
    }

    #[inline]
    pub fn get_remaining(&self) -> &'a str {
        self.remaining.get()
    }

    #[inline]
    pub fn peek(&self, n: usize) -> Option<&'a str> {
        self.remaining.get().get(..n)
    }

    #[inline]
    pub fn parse<T: Parser>(&self) -> Result<T> {
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
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Self>;
}

impl<T: Parser> Parser for Box<T> {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Box<T>> {
        Ok(Box::new(input.parse()?))
    }
}


impl Parser for Program {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Program> {
        let s = input.get_remaining();
        let mut stmts = Vec::new();

        let mut start = input.scope.start;
        for line in s.lines() {
            let end = start + line.len();

            if line.len() > 0 && !line.chars().all(|c| c.is_whitespace()) {
                let content = ParseStream::new(Span::new(start, end), line);
                stmts.push(content.parse()?);
            }
            start += line.len() + 1;
        }

        Ok(Program { stmts })
    }
}

impl Parser for Stmt {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Stmt> {
        input.parse()
            .map(|macro_def| Stmt::Macro(macro_def))
            .or_else(|_| Ok(Stmt::Expr(input.parse()?)))
    }
}

impl Parser for Macro {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Macro> {
        Ok(Macro {
            def_token: input.parse()?,
            name: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parser for Expr {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Expr> {
        input.skip_whitespace();

        let peek_stream = input.fork();
        if tokens::Lambda::parse(&peek_stream).is_ok() {
            let lamb = match input.parse() {
                Ok(lamb) => lamb,
                Err(err) => return Err(err),
            };
            Ok(Expr::Lambda(lamb))
        } else {
            let lhs = input.parse()?;
            input.skip_whitespace();
            if input.is_empty() {
                Ok(Expr::Close(lhs))
            } else {
                let rhs = input.parse()?;
                Ok(Expr::Appl(Appl { lhs, rhs }))
            }
        }
    }
}

impl Parser for Lambda {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Lambda> {
        Ok(Lambda {
            lambda_token: input.parse()?,
            var: input.parse()?,
            dot_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl Parser for Appl {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Appl> {
        Ok(Appl {
            rhs: input.parse()?,
            lhs: input.parse()?,
        })
    }
}

impl Parser for Close {
    fn parse<'a, 'tok: 'a>(input: &'a ParseStream<'tok>) -> Result<Close> {
        let lookahead = input.fork();

        Ok(if tokens::LParen::parse(&lookahead).is_ok() {
            let (content, _paren) = tokens::parse_parenthesis(input)?;
            let expr = Box::new(Expr::parse(&content)?);
            Close::Paren(expr)
        } else if tokens::Quote::parse(&lookahead).is_ok() {
            Close::Literal(input.parse()?)
        } else {
            Close::Var(input.parse()?)
        })
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
