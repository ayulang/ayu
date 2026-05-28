pub mod source;
pub mod symbol;

use std::ops::{Index, Range};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    /// The start of the span.
    pub start: usize,
    /// The exclusive end of the span.
    ///
    /// Note: A span starting at 0 and ending at 2 would only cover the indices 0 and 1.
    pub end: usize,
}

impl Span {
    /// Converts the byte offsets into a [Range].
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<usize> for Span {
    fn from(value: usize) -> Self {
        Self {
            start: value,
            end: value,
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, end): (usize, usize)) -> Self {
        Self { start, end }
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

impl From<&Span> for Range<usize> {
    fn from(value: &Span) -> Self {
        (*value).into()
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.range()]
    }
}

impl Index<&Span> for str {
    type Output = str;

    fn index(&self, index: &Span) -> &Self::Output {
        &self[*index]
    }
}
