use std::collections::VecDeque;

use crate::token::StructuredToken;

/// A stream of tokens.
pub struct TokenStream {
    tokens: VecDeque<StructuredToken>,
}

impl TokenStream {
    pub fn new(tokens: Vec<StructuredToken>) -> Self {
        Self {
            tokens: VecDeque::from(tokens),
        }
    }

    #[inline]
    pub fn consume(&mut self) -> Option<StructuredToken> {
        self.tokens.pop_front()
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
