#[derive(Debug, Clone, Copy)]
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
    pub fn start(&self) -> Span {
        Span { start: self.start, end: self.start }
    }

    #[inline]
    pub fn end(&self) -> Span {
        Span { start: self.end, end: self.end }
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
}

impl<'a> From<&'a str> for Span {
    fn from(s: &'a str) -> Self {
        Span { start: 0, end: s.len() }
    }
}

