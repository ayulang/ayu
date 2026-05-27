use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Fn,
    Let,
}

impl From<Keyword> for TokenKind {
    fn from(value: Keyword) -> Self {
        Self::Keyword(value)
    }
}
