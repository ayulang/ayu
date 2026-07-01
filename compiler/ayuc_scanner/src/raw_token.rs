use std::collections::VecDeque;

use ayuc_span::Span;

pub struct RawTokenStream(VecDeque<RawToken>);

impl RawTokenStream {
    pub fn new(tokens: Vec<RawToken>) -> Self {
        Self(tokens.into())
    }

    pub fn consume(&mut self) -> Option<RawToken> {
        self.0.pop_front()
    }

    pub fn peek(&self) -> Option<&RawToken> {
        self.0.front()
    }
}

#[derive(Debug, Clone)]
pub struct RawToken {
    /// The kind of token. Contains data.
    pub kind: RawTokenKind,

    /// The bounds of the token in the source code.
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawTokenKind {
    /// Any sequence of whitespace characters. This token is usually skipped.
    Whitespace,

    /// A comment. This token is usually skipped.
    Comment,

    /// An identifier or keyword.
    Ident,
    /// An invalid identifier.
    InvalidIdent,

    /// A literal.
    Literal {
        kind: LiteralKind,
    },

    /// ;
    Semi,
    /// :
    Colon,
    /// +
    Plus,
    /// -
    Minus,
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
    /// >
    Gt,
    /// ,
    Comma,

    /// An unknown token.
    Unknown,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    /// "abc" or "abc
    Str { terminated: bool },
    /// 12345, 00491
    Integer { data_span: Span },
}

impl RawToken {
    /// Creates a new [RawToken] with the given [RawTokenKind] and [Span].
    pub fn new<S: Into<Span>>(kind: RawTokenKind, span: S) -> Self {
        Self {
            kind,
            span: span.into(),
        }
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.kind == RawTokenKind::Eof
    }

    #[inline]
    pub fn is_whitespace(&self) -> bool {
        self.kind == RawTokenKind::Whitespace
    }
}
