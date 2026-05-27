#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}
