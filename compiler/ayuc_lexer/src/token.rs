use ayuc_span::{Span, symbol::Symbol};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// An identifier and its associated [Symbol].
    Ident(Symbol),
    /// A keywrod.
    Keyword(Keyword),
    /// A literal.
    Literal { data_span: Span },

    /// ;
    Semi,
    /// :
    Colon,
    /// =
    Equals,
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// {
    OpenBrace,
    /// }
    CloseBrace,

    /// The end of the input.
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Let,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}
