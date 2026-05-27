pub mod keyword;
pub mod literal;
pub mod punct;

use ayuc_span::Span;

use crate::token::{keyword::Keyword, literal::Literal, punct::Punct};

#[derive(Debug)]
pub struct Token {
    /// The kind of token. Contains data
    pub kind: TokenKind,

    /// The position of the token in the source code
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Literal(Literal),
    Keyword(Keyword),
    Punct(Punct),
    Eof,
}
