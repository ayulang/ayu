use ayuc_id::ast::NodeId;
use ayuc_span::Span;

use crate::Path;

#[derive(Debug)]
pub struct Ty {
    pub id: NodeId,
    pub span: Span,
    pub kind: TyKind,
}

#[derive(Debug)]
pub enum TyKind {
    Path(Path),
    Tuple(Vec<Ty>),
}

impl TyKind {
    pub const UNIT: Self = Self::Tuple(Vec::new());
}
