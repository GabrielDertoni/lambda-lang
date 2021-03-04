pub mod ast;
pub mod tokens;
pub mod error;
pub mod parser_cache;
pub mod parse_stream;

use crate::span::Span;
use ast::*;
use error::*;
pub use parse_stream::*;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Span {
    #[inline]
    fn span(&self) -> Span { *self }
}

impl<T: Spanned> Spanned for Box<T> {
    #[inline]
    fn span(&self) -> Span { self.as_ref().span() }
}

impl<T: Spanned> Spanned for [T] {
    fn span(&self) -> Span {
        self.iter()
            .map(|el| el.span())
            .fold_first(|a, b| a.merge(b))
            .unwrap()
    }
}

impl<Fst: Spanned, Snd: Spanned> Spanned for Vec<(Fst, Snd)> {
    fn span(&self) -> Span {
        let (left, right) = self.iter()
            .map(|(fst, snd)| (fst.span(), snd.span()))
            .fold_first(|(pfst, psnd), (fst, snd)| (pfst.merge(fst), psnd.merge(snd)))
            .unwrap();

        left.merge(right)
    }
}

/// Parsers need to live for 'static so that their resulting values can be
/// cached in the system. It also needs to be Clone so that it can be cloned
/// from cache inside `ParseStream`.
pub trait Parser: 'static + Clone + Spanned {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Self>;

    /// This function has a similar notation to the `Parser::parse` function,
    /// but its implementation may vary if a type can error, but still keep
    /// parsing. If it can't, then it should just use the default
    /// implementation.
    fn try_parse<'tok>(input: &ParseStream<'tok>) -> Result<Self> {
        input.parse()
    }
}

impl<T: Parser> Parser for Box<T> {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Box<T>> {
        Ok(Box::new(input.parse()?))
    }

    fn try_parse<'tok>(input: &ParseStream<'tok>) -> Result<Box<T>> {
        Ok(Box::new(input.try_parse()?))
    }
}


impl Parser for Program {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Program> {
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
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Stmt> {
        let result = input.parse()
            .map(|macro_def| Stmt::Macro(macro_def))
            .or_else(|_| {
                let cache = input.cache_borrow_mut()?;
                drop(cache);
                Ok(Stmt::Expr(input.parse()?))
            });

        if result.is_ok() && input.get_remaining().len() > 0 {
            Err(Error::new(input.curr_span(), "unexpected trailing input"))
        } else {
            result
        }
    }
}

impl Parser for Macro {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Macro> {
        Ok(Macro {
            name: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parser for Expr {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Expr> {
        input.skip_whitespace();

        Ok({
            input.parse()
                .map(|lamb| Expr::Lambda(lamb))
                .or_else(|err| {
                    input.parse()
                        .map(|appl| Expr::Appl(appl))
                        .map_err(|appl_err| err.or(appl_err))
                })
                .or_else(|err| {
                    input.parse()
                        .and_then(|close| {
                            input.skip_whitespace();

                            // At this point, it is expected to parse the entire input
                            if let None = input.get() {
                                Ok(Expr::Close(close))
                            } else {
                                Err(Error::new(input.curr_span().start(), "unexpected trailing input"))
                            }
                        })
                        .map_err(|close_err| err.or(close_err))
                })
                .map_err(|err| {
                    Error::new(err.cover_span(), "expected an expression")
                })
        }?)
    }
}

impl Parser for Lambda {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Lambda> {
        Ok(Lambda {
            lambda_token: input.parse()?,
            var: input.parse()?,
            dot_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl Parser for Appl {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Appl> {
        let lo = input.curr_span().start;
        let mut root = Appl {
            lhs: input.parse()?,
            rhs: input.parse()?,
        };

        while !input.is_empty() {
            let rhs = input.parse()?;
            let hi = input.curr_span().start;
            let group = tokens::Group::new(Span::new(lo, hi), tokens::Delimiter::None);
            root = Appl {
                lhs: Close::Grouping(box Expr::Appl(root), group),
                rhs,
            };
        }
        Ok(root)

        // parse_appl_with_lhs(input, lhs)
    }
}

/*
fn parse_appl_with_lhs<'tok>(input: &ParseStream<'tok>, lhs: Close, mut log: usize) -> Result<Appl> {
        let mut root = Appl {
            lhs,
            rhs: input.parse()?,
        };

        while !input.is_empty() {
            let rhs = input.parse()?;
            let hi = input.curr_span().start;
            let group = tokens::Group::new(Span::new(lo, hi), tokens::Delimiter::None);
            root = Appl {
                lhs: Close::Grouping(box Expr::Appl(root), ),
                rhs,
            };
        }
        Ok(root)
}
*/

impl Parser for Close {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Close> {
        Ok({
            input
                .parse_parethesized()
                .map(|(expr, group)| {
                    Close::Grouping(expr, tokens::Group::new(group, tokens::Delimiter::Paren))
                })
                .or_else(|err| {
                    input.parse()
                        .map(|var| Close::Var(var))
                        .map_err(|var_err| {
                            err.or(var_err)
                            /*
                            error.extend(var_err.messages);
                            Err(error)
                            */
                        })
                })
                .or_else(|err| {
                    input.parse()
                        .map(|lit| Close::Literal(lit))
                        .map_err(|lit_err| {
                            err.or(lit_err)
                            /*
                            error.extend(var_err.messages);
                            Err(error)
                            */
                        })
                })
        }?)
    }
}

impl Parser for VarList {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Self> {
        input.skip_whitespace();

        let mut list = Vec::new();
        while let Ok(tuple) = input.parse_once(|s| Ok((s.parse()?, s.parse()?))) {
            list.push(tuple);
        }

        if list.len() > 0 {
            Ok(VarList { list })
        } else {
            Err(Error::new(input.curr_span().start(), "expected variable list"))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_multiple_appl() {
        let stream = ParseStream::from("(\\a. a) (\\a. a) \"hello\"");
        assert!(Appl::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.get_remaining());
    }

    #[test]
    fn test_parse_stmt() {
        let stream = ParseStream::from("\\a. a a");
        assert!(Stmt::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.get_remaining());
    }

    #[test]
    fn test_literal_parser() {
        let stream = ParseStream::from("\\a. a a");
        assert!(Expr::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.get_remaining());
    }

    #[test]
    fn test_paren() {
        let stream = ParseStream::from("(\\a. a a)");
        assert!(Expr::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.get_remaining());
    }

    #[test]
    fn test_var() {
        let stream = ParseStream::from("a");
        assert!(tokens::Var::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.get_remaining());
    }

    #[test]
    fn test_literal() {
        let stream = ParseStream::from("\"hello world\"");
        assert!(tokens::Literal::parse(&stream).is_ok());
        assert!(stream.is_empty(), "remaining: {}", stream.get_remaining());
    }
}
