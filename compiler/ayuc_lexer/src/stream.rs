use ayuc_span::Span;

use crate::token::StructuredToken;

#[derive(Debug, Clone, Copy)]
pub struct Snapshot(usize);

/// A stream of tokens.
pub struct TokenStream {
    tokens: Vec<StructuredToken>,
    pos: usize,
}

impl TokenStream {
    pub const fn new(tokens: Vec<StructuredToken>) -> Self {
        Self {
            tokens: tokens,
            pos: 0,
        }
    }

    pub const fn snapshot(&self) -> Snapshot {
        Snapshot(self.pos)
    }

    #[inline]
    pub fn restore(&mut self, snapshot: Snapshot) {
        self.pos = snapshot.0;
    }

    pub fn consume(&mut self) -> Option<&StructuredToken> {
        if let Some(token) = self.tokens.get(self.pos) {
            self.pos += 1;

            Some(token)
        } else {
            None
        }
    }

    pub fn span_since(&self, snapshot: Snapshot) -> Span {
        if snapshot.0 < self.pos {
            let start = self.tokens[snapshot.0].span().start;
            let end = self.tokens[self.pos - 1].span().end;

            Span::from(start..end)
        } else {
            let start = self
                .tokens
                .get(snapshot.0)
                .map(|t| t.span().start)
                .unwrap_or(0);

            Span::from(start..start)
        }
    }

    #[inline]
    pub fn past_span(&self, nth: usize) -> Option<Span> {
        self.tokens.get(self.pos - nth).map(|t| t.span())
    }

    #[inline]
    pub fn past_span_or_distance(&self, nth: usize, snapshot: Snapshot) -> Span {
        self.past_span(nth)
            .unwrap_or_else(|| self.span_since(snapshot))
    }

    #[inline]
    pub fn first(&self) -> Option<&StructuredToken> {
        self.tokens.get(self.pos)
    }

    #[inline]
    pub fn second(&self) -> Option<&StructuredToken> {
        self.tokens.get(self.pos + 1)
    }

    #[inline]
    pub fn third(&self) -> Option<&StructuredToken> {
        self.tokens.get(self.pos + 2)
    }
}
