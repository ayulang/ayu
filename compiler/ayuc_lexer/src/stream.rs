use std::collections::VecDeque;

use crate::token::Token;

/// A stream of tokens.
pub struct TokenStream {
    tokens: VecDeque<Token>,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: VecDeque::from(tokens),
        }
    }

    #[inline]
    pub fn consume(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    #[inline]
    pub fn first(&self) -> Option<&Token> {
        self.tokens.get(0)
    }

    #[inline]
    pub fn second(&self) -> Option<&Token> {
        self.tokens.get(1)
    }

    #[inline]
    pub fn third(&self) -> Option<&Token> {
        self.tokens.get(2)
    }
}
