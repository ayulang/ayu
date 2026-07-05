use ayuc_id::ast::NodeId;
use ayuc_span::Span;

use crate::Ident;

#[derive(Debug)]
pub struct Path {
    pub span: Span,
    pub segments: Vec<PathSegment>,
}

#[derive(Debug)]
pub struct PathSegment {
    pub id: NodeId,
    pub ident: Ident,
}

impl Path {
    pub fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.ident.sym.as_str())
            .collect::<Vec<_>>()
            .join("::")
    }
}
