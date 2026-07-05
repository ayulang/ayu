use std::fmt::Display;

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

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }

            write!(f, "{}", segment.ident.sym)?;
        }

        Ok(())
    }
}
