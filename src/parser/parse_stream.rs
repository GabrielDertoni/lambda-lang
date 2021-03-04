use std::any::TypeId;
use std::collections::HashMap;
use std::cell::Cell;
use std::rc::Rc;
use std::cell::{ RefCell, RefMut };
use std::str::pattern::Pattern;

use crate::span::*;
use super::{ Parser, Result };
use super::error::Error;
use super::parser_cache::{ ParserCache, ParsedType };

#[derive(Clone)]
pub struct ParseStream<'a> {
    pub scope: Span,
    curr_span: Cell<Span>,
    cache: Rc<RefCell<ParserCache>>,
    original: &'a str,
    remaining: Cell<&'a str>,
    error: RefCell<Option<Error>>,
}

impl<'a> ParseStream<'a> {
    pub fn new(scope: Span, s: &'a str) -> ParseStream<'a> {
        ParseStream {
            scope,
            curr_span: Cell::new(scope),
            cache: Rc::new(RefCell::new(ParserCache::new())),
            original: s,
            remaining: Cell::new(s),
            error: RefCell::new(None),
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

    /// Moves the stream to some index that represents the start of the current
    /// span. This can be used to quickly advance the stream to a certain span.
    pub fn goto(&self, i: usize) {
        assert!(i <= self.scope.end, "tried to go to byte {} but scope is {:?}", i, self.scope);
        self.curr_span.set(Span::new(i, self.scope.end));

        // Adjust the byte index to the local scope.
        self.remaining.set(&self.original[i - self.scope.start..]);
    }

    pub fn goto_remaining(&self, n: usize) {
        let len = self.original.len();
        self.curr_span.set(Span::new(len - n, len));
        self.remaining.set(&self.original[len - n..]);
    }

    // Advances the stream, skipping any space, and returns the next
    // non-whitespace char.
    pub fn next(&self) -> Option<char> {
        self.skip_whitespace();
        self.get()
    }

    pub fn get_line_column_number(&self, span: Span) -> (usize, usize) {
        for (i, line) in self.line_spans().into_iter().enumerate() {
            if line.contains(span.start()) {
                return (i, span.start - line.start)
            }
        }
        panic!("Unable to get line and column number")
    }

    fn line_spans(&self) -> Vec<Span> {
        let mut span = Span::new(0, self.original.len());
        let mut lines = Vec::new();

        for line in self.original.lines() {
            lines.push(span.with_width(line.len()));
            span.start += line.len() + 1;
        }

        lines
    }

    pub fn cache_borrow_mut(&self) -> Result<RefMut<ParserCache>> {
        self.cache.as_ref()
            .try_borrow_mut()
            .map_err(|_|
                Error::new_compiler_err(
                    "failed to borrow mutably the parse cache."
                )
            )
    }

    // Tries to parse usign some function. If it is successfull, it adds a copy
    // of the value to a cache. If somehere down the line the parsing fails,
    // but it had already parsed some correct values, those values may can be
    // directly cloned from cache and no additional parsing needs to occur.
    // This avoids needles backtracking using essentially a momoization approach.
    // Do note that the when cloning from cache, there may be overhead if the
    // AST is very large.
    fn parse_with<T, F>(&self, mut parse_fn: F) -> Result<T>
    where
        T: 'static + Clone,
        F: FnMut(&ParseStream) -> Result<T>,
    {
        let type_id = TypeId::of::<T>();
        let remaining_len = self.curr_span().start;
        let mut cache_ref = self.cache_borrow_mut()?;

        Ok(match cache_ref.get_mut(&remaining_len) {
            Some(ref cached)
                if let Some(found) = cached.get(&type_id) => {
                    let success = found.as_ref().map_err(|err| err.clone())?;
                    self.goto(success.parses_until);
                    AsRef::<T>::as_ref(success).clone()
            },
            _ => {
                // Needs to be dropped here so we can call T::parse() which may
                // call this function recursivelly.
                drop(cache_ref);

                let parse_result = parse_fn(self)
                    .map(|parsed| ParsedType::new(self.curr_span().start, parsed));

                // Borrow again after T::parse() used it.
                let mut cache_ref = self.cache_borrow_mut()?;

                // This means we have to perform two `cache_ref.get_mut()`.
                let map = match cache_ref.get_mut(&remaining_len) {
                    Some(map) => map,
                    None => {
                        let new_map = HashMap::new();
                        assert!(cache_ref.insert(remaining_len, new_map).is_none());
                        cache_ref
                            .get_mut(&remaining_len)
                            .unwrap() // Safe: we have just inserted the entry.
                    }
                };
                assert!(map.insert(type_id, parse_result).is_none());
                let just_inserted = map.get(&type_id)
                    .unwrap()
                    .as_ref()
                    .map_err(|err| err.clone())?;

                AsRef::<T>::as_ref(just_inserted).clone()
            },
        })
    }

    pub fn parse<T: Parser>(&self) -> Result<T> {
        let lookahead = self.fork();
        let val = lookahead.parse_with(T::parse)?;
        self.merge(lookahead);
        Ok(val)
    }

    /// Tries to parse a value T from the stream. If it can, it will be returned
    /// with `Ok`, if it can't it may still be able to return `Ok`, but then the
    /// `ParseStream` will have some errors in its `error` field. If there is
    /// no way to 
    pub fn try_parse<T: Parser>(&self) -> Result<T> {
        self.parse_with(T::try_parse)
    }

    pub fn parse_once<T, F>(&self, f: F) -> Result<T>
    where
        F: Fn(&ParseStream<'a>) -> Result<T>,
    {
        let lookahead = self.fork();
        let val = f(&lookahead)?;
        self.merge(lookahead);
        Ok(val)
    }

    pub fn parse_enclosed<T: Parser>(&self, open: &str, close: &str) -> Result<(T, Span)> {
        let (stream, span) = parse_enclosed(self, open, close)?;
        let val = stream.parse_with(T::parse)?;
        self.goto(span.end);

        Ok((val, span))
    }

    pub fn parse_parethesized<T: Parser>(&self) -> Result<(T, Span)> {
        self.parse_enclosed("(", ")")
    }


    /// Removes errors from the stream and reaturn them if they exist.
    pub fn assert_no_errors(&self) -> Result<()> {
        match self.error.take() {
            Some(err) => Err(err),
            None      => Ok(()),
        }
    }

    pub fn error(&self, err: Error) {
        let mut borrow = self.error.borrow_mut();
        match borrow.as_mut() {
            Some(local) => local.extend(err.messages),
            None                => { borrow.replace(err); },
        }
    }

    #[inline]
    fn merge(&self, other: ParseStream<'a>) {
        // Make sure both point to the exact same original string.
        assert!(self.original.as_ptr() == other.original.as_ptr());
        assert!(Rc::ptr_eq(&self.cache, &other.cache));
        self.remaining.set(other.get_remaining());
        self.curr_span.set(other.curr_span());
    }

    fn new_child(&self, scope: Span, s: &'a str) -> ParseStream<'a> {
        ParseStream {
            scope,
            curr_span: Cell::new(scope),
            cache: Rc::clone(&self.cache),
            original: s,
            remaining: Cell::new(s),
            error: RefCell::new(None),
        }
    }

    #[inline]
    fn fork(&self) -> ParseStream<'a> {
        self.clone()
    }

    #[inline]
    pub fn get(&self) -> Option<char> {
        self.get_remaining().chars().nth(0)
    }

    pub fn starts_with<'b, P: Pattern<'b>>(&'b self, patt: P) -> bool {
        self.get_remaining().starts_with(patt)
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
    pub fn is_empty(&self) -> bool {
        self.get_remaining()
            .chars()
            .all(char::is_whitespace)
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

fn parse_enclosed<'tok>(input: &ParseStream<'tok>, open: &str, close: &str) -> Result<(ParseStream<'tok>, Span)> {
    assert!(open != "\"" && close != "\"");
    input.skip_whitespace();

    let mut remaining = input.get_remaining();
    let start = input.curr_span().start;

    match remaining.get(..1) {
        Some(fst) if fst == open => (),
        _         => return Err(Error::new(input.curr_span().start(), "expected a '('")),
    }

    let mut count = 1;
    let mut prev = &remaining[..1];
    let mut str_start = None;

    // Skips the first '('
    remaining = &remaining[1..];

    let mut unclosed = vec![start];

    while unclosed.len() > 0 && remaining.len() > 0 {
        let c = &remaining[..1];
        remaining = &remaining[1..];

        if str_start.is_none() {
            if c == open {
                unclosed.push(start + count);
            } else if c == close {
                unclosed.pop();
            }
        }

        if c == "\"" {
            if str_start.is_some() && prev != "\\" {
                str_start = None;
            } else {
                str_start = Some(start + count);
            }
        }
        prev = c;
        count += 1;
    }

    if let Some(open_quote) = str_start {
        return Err(Error::new(Span::new_start(open_quote), "unmatched quote"))
    }

    if let Some(open_paren) = unclosed.pop() {
        return Err(Error::new(Span::new_start(open_paren), "unmatched parenthesis"));
    }

    let inner_start = start + 1;
    let inner_end = start + count - 1;
    let stream = input.new_child(
        (inner_start..inner_end).into(),
        &input.original[inner_start - input.scope.start..inner_end - input.scope.start]
    );

    Ok((stream, (start..start + count).into()))
}
