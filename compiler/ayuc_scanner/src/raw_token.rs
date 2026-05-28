use ayuc_span::Span;

#[derive(Debug)]
pub struct RawToken {
    /// The kind of token. Contains data.
    pub kind: RawTokenKind,

    /// The bounds of the token in the source code.
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawTokenKind {
    /// Any sequence of whitespace characters.
    Whitespace,
    /// An identifier or keyword.
    Ident,
    /// An invalid identifier.
    InvalidIdent,
    /// A literal.
    Literal {
        kind: LiteralKind,
    },

    /// An unknown token.
    Unknown,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    /// "abc" or "abc
    Str { terminated: bool },
}

impl RawToken {
    /// Creates a new [RawToken] with the given [RawTokenKind] and [Span].
    pub fn new<S: Into<Span>>(kind: RawTokenKind, span: S) -> Self {
        Self {
            kind,
            span: span.into(),
        }
    }
}
