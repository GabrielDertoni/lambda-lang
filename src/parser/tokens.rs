use super::{ Parser, Result, ParseStream };
use super::error::Error;
use super::span::Span;

macro_rules! define_token_struct {
    (pub struct $tok:ident) => {
        #[derive(Debug)]
        pub struct $tok {
            pub span: Span,
        }

        impl $tok {
            fn new(span: Span) -> Self {
                $tok { span }
            }
        }
    }
}

macro_rules! define_token_structs {
    () => {};

    (once $($patt:literal)|+ => pub struct $tok:ident, $($rest:tt)*) => {
        define_token_struct!(pub struct $tok);

        impl Parser for $tok {
            fn parse<'a>(input: &'a ParseStream<'a>) -> Result<$tok> {
                input.skip_whitespace();

                let span = input.curr_span();
                if let $(Some($patt))|+ = input.get() {
                    input.advance();
                    Ok($tok::new(span.with_width(1)))
                } else {
                    Err(Error::new(span.start(), "Error, expected token Dot"))
                }
            }
        }

        define_token_structs! { $($rest)* }
    };

    (many $($patt:literal)|+ => pub struct $tok:ident, $($rest:tt)*) => {
        define_token_struct!(pub struct $tok);

        impl Parser for $tok {
            fn parse<'a>(input: &'a ParseStream<'a>) -> Result<$tok> {
                input.skip_whitespace();

                let mut count = 0;
                let mut span = input.curr_span();

                while let $(Some($patt))|+ = input.get() {
                    input.advance();

                    span = span.merge(input.curr_span());
                    count += 1;
                }

                if count > 0 {
                    Ok($tok::new(span.with_width(count)))
                } else {
                    Err(Error::new(span.start(), format!("Error, expected token {}", stringify!($tok))))
                }
            }
        }

        define_token_structs! { $($rest)* }
    }
}

define_token_structs! {
    many '\n'       => pub struct Ln,
    many ' '        => pub struct Space,
    once '.'        => pub struct Dot,
    once '='        => pub struct Equal,
    once '('        => pub struct LParen,
    once ')'        => pub struct RParen,
    once '"'        => pub struct Quote,
    once '\\' | 'λ' => pub struct Lambda,
    once '$'        => pub struct EOF,
}

#[derive(Debug)]
pub struct Var {
    pub span: Span,
    pub name: String,
}

impl Var {
    fn new(span: Span, name: String) -> Var {
        Var { span, name }
    }
}

#[derive(Debug)]
pub struct Literal {
    pub span: Span,
    pub content: String,
}

impl Literal {
    fn new(span: Span, content: String) -> Literal {
        Literal { span, content }
    }
}

pub struct Paren {
    pub span: Span,
}

impl Paren {
    fn new(span: Span) -> Paren {
        Paren { span }
    }
}

fn skip_string<'a>(input: &'a ParseStream<'a>) -> usize {
    let mut count = 0;
    assert!(input.get().unwrap() == '"');
    while let Some(c) = input.get() {
        // If it is an escape char, advance one more
        if c == '\\' {
            input.advance();
            count += 1;
        } else if c == '"' {
            break;
        }
        input.advance();
        count += 1;
    }
    count
}

fn skip_until_paren<'a>(input: &'a ParseStream<'a>) -> usize {
    let mut count = 0;
    while let Some(c) = input.get() {
        if c == '(' || c == ')' {
            break;
        } else if c == '"' {
            count += skip_string(input);
        }
        input.advance();
        count += 1;
    }
    count
}

pub fn parse_parenthesis<'a>(input: &'a ParseStream<'a>) -> Result<(ParseStream<'a>, Paren)> {
    let mut depth = 0;
    let original = input.get_remaining();
    let start = input.curr_span().start();
    assert!(input.get().unwrap() == '(');
    while let Some(c) = input.get() {
        if c == '(' {
            depth += 1;
        } else {
            depth -= 1;
        }

        if depth == 0 {
            break;
        } else if depth < 0 {
            return Err(Error::new(input.curr_span().start(), "Unmatched parenthesis"));
        }

        skip_until_paren(input);
    }

    let span = start.merge(input.curr_span().start());
    let stream = ParseStream::new(span, &original[1..span.width()-2]);
    let paren = Paren::new(span);
    Ok((stream, paren))
}

impl Parser for Var {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Var> {
        input.skip_whitespace();
        let span = input.curr_span();
        let mut content = String::new();

        while let Some(c) = input.get() {
            if c.is_alphabetic() {
                content.push(c);
            } else {
                break;
            }
            input.advance();
        }
        if content.len() == 0 {
            Err(Error::new(span.start(), "Expected a variable"))
        } else {
            Ok(Var::new(span.with_width(content.len()), content))
        }
    }
}

impl Parser for Literal {
    fn parse<'a>(input: &'a ParseStream<'a>) -> Result<Literal> {
        input.skip_whitespace();
        let span = input.curr_span();
        let mut content = String::new();
        let mut count = 0;

        Quote::parse(input)?;
        while let Some(c) = input.get() {
            if c == '\\' {
                input.advance();
                input.skip_whitespace();
                if let Some(escaped) = input.get() {
                    content.push(escaped);
                } else {
                    return Err(Error::new(input.curr_span().start(), "Escape without escaped"));
                }
            } else if c == '"' {
                break;
            } else {
                content.push(c);
            }
            input.advance();
            input.skip_whitespace();
            count += 1;
        }
        Quote::parse(input)?;

        Ok(Literal::new(span.with_width(count), content))
    }
}

