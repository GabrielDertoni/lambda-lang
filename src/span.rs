use std::fmt;

const DUMMY_SPAN: Span = Span { start: 0, end: 0 };

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    #[inline]
    pub fn new_start(start: usize) -> Span {
        Span { start, end: start + 1 }
    }

    #[inline]
    pub fn start(&self) -> Span {
        Span { start: self.start, end: self.start + 1 }
    }

    #[inline]
    pub fn end(&self) -> Span {
        Span { start: self.end - 1, end: self.end }
    }

    #[inline]
    pub fn merge(&self, other: Span) -> Span {
        use std::cmp::{ min, max };
        Span {
            start: min(self.start, other.start),
            end: max(self.end, other.end),
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    pub fn with_width(&self, width: usize) -> Span {
        assert!(self.width() >= width);
        Span { start: self.start, end: self.start + width }
    }

    #[inline]
    pub fn contains(&self, other: Span) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn into_range(self) -> Range<usize> {
        self.into()
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bytes({}..{})", self.start, self.end)
    }
}

impl<'a> From<&'a str> for Span {
    fn from(s: &'a str) -> Self {
        Span { start: 0, end: s.len() }
    }
}

use std::ops::Range;

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Span {
        Span::new(range.start, range.end)
    }
}

impl Into<Range<usize>> for Span {
    fn into(self) -> Range<usize> {
        self.start..self.end
    }
}

use std::cmp::{ Ord, PartialOrd, Ordering };

impl PartialOrd for Span {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Span {
    // The first criteria is the start of the span, the second is the width.
    fn cmp(&self, other: &Self) -> Ordering {
        if self.start > other.start {
            Ordering::Greater
        } else if self.start < other.start {
            Ordering::Less
        } else if self.width() > other.width() {
            Ordering::Greater
        } else if self.width() < other.width() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

