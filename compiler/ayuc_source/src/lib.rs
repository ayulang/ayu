pub mod cache;

use std::ops::{Deref, DerefMut, Index};

use ayuc_span::Span;

use crate::cache::FileId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceSpan {
    pub file_id: FileId,

    span: Span,
}

impl SourceSpan {
    pub const fn new(file_id: usize, span: Span) -> Self {
        Self { file_id, span }
    }

    pub const fn as_span(&self) -> Span {
        self.span
    }
}

impl Deref for SourceSpan {
    type Target = Span;

    fn deref(&self) -> &Self::Target {
        &self.span
    }
}

impl DerefMut for SourceSpan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.span
    }
}

impl ariadne::Span for SourceSpan {
    type SourceId = usize;

    fn source(&self) -> &Self::SourceId {
        &self.file_id
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl Index<SourceSpan> for str {
    type Output = str;

    fn index(&self, index: SourceSpan) -> &Self::Output {
        &self[index.span]
    }
}
