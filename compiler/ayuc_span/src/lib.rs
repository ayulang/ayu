pub mod symbol;

use std::{
    fmt::Debug,
    ops::{Index, Range},
};

#[derive(Clone, Copy, PartialEq)]
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

    /// Merges this span with another span.
    ///
    /// Example: Mergin (0..10) with (10..15) would create (0..15)
    #[inline]
    pub fn merge(&mut self, other: Span) {
        if self.start > other.start {
            self.start = other.start;
        }

        if self.end < other.end {
            self.end = other.end;
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        0.into()
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Span({}..{})", self.start, self.end)
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

impl From<Range<usize>> for Span {
    fn from(Range { start, end }: Range<usize>) -> Self {
        Self { start, end }
    }
}

impl From<&Span> for Span {
    fn from(value: &Span) -> Self {
        *value
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
