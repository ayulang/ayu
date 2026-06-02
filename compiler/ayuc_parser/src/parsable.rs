use ayuc_span::Span;

use crate::{Parser, session::ParseSession};

pub trait Parsable: Sized {
    /// Tries to parse this node.
    ///
    /// ## Returns
    ///
    /// - `Err`, if any error occurred and the node couldn't be parsed.
    /// - `Ok`, if the node was parsed successfully.
    fn parse<'a>(parser: &mut Parser<'a>, sess: &mut ParseSession<'a>) -> Result<Parsed<Self>, ()>;
}

pub enum Parsed<T> {
    Present(T),
    Missing(Span),
}

impl<T> Parsed<T> {
    pub fn is_missing(&self) -> bool {
        match self {
            Self::Missing(_) => true,
            _ => false,
        }
    }
}
