use ayuc_span::Span;

use crate::Parser;

pub trait Parsable: Sized {
    /// Tries to parse this node.
    ///
    /// ## Returns
    ///
    /// - `Err`, if any error occurred and the node couldn't be parsed.
    /// - `Ok`, if the node was parsed successfully.
    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError>;
}

pub trait Assertable: Parsable {
    /// The name of the parsable for automatic diagnostic creation.
    const NAME: &str;
}

#[derive(Debug)]
pub enum ParseError {
    Unrecoverable,
}

#[derive(Debug)]
pub enum Parsed<T> {
    Present(T),
    Missing(Span),
}

impl<T> Parsed<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing(_))
    }

    pub fn unwrap_or(self, value: T) -> T {
        match self {
            Self::Present(p) => p,
            Self::Missing(_) => value,
        }
    }
}
