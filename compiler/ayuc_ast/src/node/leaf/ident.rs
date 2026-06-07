use ayuc_span::{Span, symbol::Symbol};

#[derive(Debug)]
pub struct Ident {
    pub sym: Symbol,
    pub span: Span,
}
