use ayuc_id::ast::NodeId;
use ayuc_span::{Span, symbol::Symbol};

#[derive(Debug)]
pub struct Pat {
    pub span: Span,
    pub id: NodeId,
    pub kind: PatKind,
}

#[derive(Debug)]
pub enum PatKind {
    Binding(PatBinding),
    Tuple(Vec<Pat>),
}

#[derive(Debug)]
pub struct PatBinding {
    pub sym: Symbol,
    pub mutable: bool,
}
