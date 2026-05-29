pub mod cache;

use ayuc_span::Span;

use crate::cache::FileId;

#[derive(Debug, Clone, Copy)]
pub struct SourceSpan {
    pub file_id: FileId,
    pub span: Span,
}

impl SourceSpan {
    pub const fn new(file_id: usize, span: Span) -> Self {
        Self { file_id, span }
    }
}

impl ariadne::Span for SourceSpan {
    type SourceId = usize;

    fn source(&self) -> &Self::SourceId {
        &self.file_id
    }

    fn start(&self) -> usize {
        self.span.start
    }

    fn end(&self) -> usize {
        self.span.end
    }
}
