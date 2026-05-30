use std::collections::VecDeque;

use ayuc_span::Span;

use crate::token::StructuredToken;

/// A stream of tokens.
pub struct TokenStream {
    tokens: VecDeque<StructuredToken>,

    pub last_position: Span,
}

impl TokenStream {
    pub fn new(tokens: Vec<StructuredToken>) -> Self {
        let span = tokens[0].span();

        Self {
            tokens: VecDeque::from(tokens),
            last_position: (span.start).into(),
        }
    }

    #[inline]
    pub fn consume(&mut self) -> Option<StructuredToken> {
        match self.tokens.pop_front() {
            Some(tok) => {
                self.last_position = tok.span();

                Some(tok)
            }
            None => None,
        }
    }

    #[inline]
    pub fn first(&self) -> Option<&StructuredToken> {
        self.tokens.get(0)
    }

    #[inline]
    pub fn second(&self) -> Option<&StructuredToken> {
        self.tokens.get(1)
    }

    #[inline]
    pub fn third(&self) -> Option<&StructuredToken> {
        self.tokens.get(2)
    }
}
