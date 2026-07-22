use ayuc_id::ast::NodeId;
use ayuc_span::{Span, symbol::Symbol};

#[derive(Debug)]
pub struct LocalInfo {
    pub name: Symbol,
    pub defined_where: Span,
    pub id: NodeId,
    pub mutable: bool,
}
