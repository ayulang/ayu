use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String { data: String, quote_kind: QuoteKind },
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteKind {
    /// A double quote: "
    Double,
}

impl From<Literal> for TokenKind {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}
