use std::ops::Index;

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
    /// Calculates the length of the span.
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
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

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}
