use super::{ Parser, Spanned, Result, ParseStream };
use super::error::Error;
use crate::span::Span;

macro_rules! define_token_structs {
    () => {};

    (pub struct $tok:ident, $($rest:tt)*) => {
        #[derive(Debug, Clone)]
        pub struct $tok {
            pub span: Span,
        }

        impl $tok {
            pub fn new(span: Span) -> Self {
                $tok { span }
            }
        }

        impl Spanned for $tok {
            fn span(&self) -> Span {
                self.span
            }
        }
        
        define_token_structs! { $($rest)* }
    };
}

macro_rules! define_token_rules {
    () => {};

    ($($patt:literal)|+ => pub struct $tok:ident, $($rest:tt)*) => {
        define_token_structs!(pub struct $tok,);

        impl Parser for $tok {
            fn parse<'tok>(input: &ParseStream<'tok>) -> Result<$tok> {
                input.skip_whitespace();

                let span = input.curr_span();
                let patts: &[&str] = &[$($patt),+];
                if let Some(patt) = patts.iter().find(|&p| input.starts_with(p)) {
                    input.advance_by(patt.len());
                    Ok($tok::new(span.with_width(1)))
                } else {
                    Err(Error::new(span.start(), format!("Error, expected token {}", stringify!($tok))))
                }
            }
        }

        define_token_rules! { $($rest)* }
    };

    ($patt:literal* => pub struct $tok:ident, $($rest:tt)*) => {
        define_token_structs!(pub struct $tok,);

        impl Parser for $tok {
            fn parse<'tok>(input: &ParseStream<'tok>) -> Result<$tok> {
                input.skip_whitespace();

                let mut count = 0;
                let mut span = input.curr_span();

                while let Some($patt) = input.get() {
                    input.advance();

                    span = span.merge(input.curr_span());
                    count += 1;
                }

                if count > 0 {
                    Ok($tok::new(span.with_width(count)))
                } else {
                    Err(Error::new(span.start(), format!("expected token {}", stringify!($tok))))
                }
            }
        }

        define_token_rules! { $($rest)* }
    };
}

define_token_rules! {
    '\n'*      => pub struct Ln,
    ' '*       => pub struct Space,
    "."        => pub struct Dot,
    "="        => pub struct Equal,
    "("        => pub struct LParen,
    ")"        => pub struct RParen,
    "\""       => pub struct Quote,
    "\\" | "λ" => pub struct Lambda,
    "$"        => pub struct EOF,
    "def"      => pub struct Def,
}

define_token_structs! {
    pub struct Paren,
}

#[derive(Debug, Clone)]
pub enum Delimiter {
    Paren,
    None,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub delim: Delimiter,
    pub span: Span,
}

// TODO: Change struct name to Ident
#[derive(Debug, Clone)]
pub struct Var {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub span: Span,
    pub content: String,
}

impl Group {
    pub fn new(span: Span, delim: Delimiter) -> Group {
        Group { span, delim }
    }

    pub fn new_unmarked(span: Span) -> Group {
        Group { span, delim: Delimiter::None }
    }
}

impl Spanned for Group {
    fn span(&self) -> Span {
        self.span
    }
}

impl Var {
    pub fn new(span: Span, name: String) -> Var {
        Var { span, name }
    }
}

impl Spanned for Var {
    fn span(&self) -> Span {
        self.span
    }
}

impl Literal {
    pub fn new(span: Span, content: String) -> Literal {
        Literal { span, content }
    }
}

impl Spanned for Literal {
    fn span(&self) -> Span {
        self.span
    }
}

fn skip_string<'tok>(input: &ParseStream<'tok>) -> usize {
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

fn skip_until_paren<'tok>(input: &ParseStream<'tok>) -> usize {
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

pub fn parse_parenthesis<'tok>(input: &ParseStream<'tok>) -> Result<(ParseStream<'tok>, Paren)> {
    input.skip_whitespace();

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
        input.advance();

        if depth == 0 {
            break;
        } else if depth < 0 {
            return Err(Error::new(input.curr_span().start(), "Unmatched parenthesis"));
        }

        skip_until_paren(input);
    }

    let span = start.merge(input.curr_span().start());
    let stream = ParseStream::new(span, &original[1..span.width() - 1]);
    let paren = Paren::new(span);
    Ok((stream, paren))
}

impl Parser for Var {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Var> {
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
            Err(Error::new(span.start(), "Expected an identifier"))
        } else {
            Ok(Var::new(span.with_width(content.len()), content))
        }
    }

    /*
    fn try_parse<'tok>(input: &ParseStream<'tok>) -> Result<Var> {
        let special: &[char] = &['\'', '_', '=', '+', '#'];
        match input.parse() {
            Ok(v)    => Ok(v),
            Err(err) => {
                if let Some(c) = input.get() {
                    if c.is_numeric() || special.iter().any(|&m| m == c) {
                        input.advance();

                        // TODO: Remove this recursive call.
                        return input.try_parse();
                    }
                }
                Err(err)
            }
        }
    }
    */
}

impl Parser for Literal {
    fn parse<'tok>(input: &ParseStream<'tok>) -> Result<Literal> {
        input.skip_whitespace();
        let span = input.curr_span();
        let mut content = String::new();
        let mut count = 0;

        Quote::parse(input)?;
        while let Some(c) = input.get() {
            if c == '\\' {
                input.advance();
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
            count += 1;
        }
        Quote::parse(input)?;

        Ok(Literal::new(span.with_width(count), content))
    }
}

