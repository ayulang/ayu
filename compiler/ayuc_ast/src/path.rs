use ayuc_id::ast::NodeId;
use ayuc_span::Span;

use crate::Ident;

#[derive(Debug)]
pub struct Path {
    pub span: Span,
    pub segements: Vec<PathSegment>,
}

#[derive(Debug)]
pub struct PathSegment {
    pub id: NodeId,
    pub ident: Ident,
}
