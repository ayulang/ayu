use crate::token::TokenKind;

/// Represents the kind of punctuation token
#[derive(Debug, Clone, PartialEq)]
pub enum Punct {
    /// ;
    Semi,
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// ->
    Arrow,
}

impl From<Punct> for TokenKind {
    fn from(value: Punct) -> Self {
        Self::Punct(value)
    }
}
