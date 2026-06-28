use std::fmt::Display;

use ayuc_span::{Span, symbol::Symbol};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructuredToken {
    Token(Token),
    Delimited(Span, Delimiter, Vec<StructuredToken>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    /// ( )
    Parenthesis,
    /// { }
    Braces,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str { data_span: Span },
    Integer { data_span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// An identifier and its associated [Symbol].
    Ident(Symbol),
    /// A keywrod.
    Keyword(Keyword),
    /// A literal.
    Literal(Literal),

    /// ;
    Semi,
    /// :
    Colon,
    /// ::
    DoubleColon,
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
    /// ->
    Arrow,
    /// ,
    Comma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Let,
    Extern,
    Return,
    If,
}

impl StructuredToken {
    pub fn span(&self) -> Span {
        match self {
            StructuredToken::Token(t) => t.span,
            Self::Delimited(span, ..) => *span,
        }
    }
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl TokenKind {
    pub fn to_delimiter(&self) -> Option<Delimiter> {
        match self {
            Self::OpenParen | Self::CloseParen => Some(Delimiter::Parenthesis),
            Self::OpenBrace | Self::CloseBrace => Some(Delimiter::Braces),
            _ => None,
        }
    }
}

impl Delimiter {
    pub fn closing_kind(&self) -> TokenKind {
        match self {
            Self::Parenthesis => TokenKind::CloseParen,
            Self::Braces => TokenKind::CloseBrace,
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kw = match self {
            Self::Fn => "fn",
            Self::Let => "let",
            Self::Extern => "extern",
            Self::Return => "return",
            Self::If => "if",
        };

        write!(f, "{kw}")
    }
}
