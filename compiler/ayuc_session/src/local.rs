use ayuc_id::ast::NodeId;
use ayuc_span::{Span, symbol::Symbol};

pub struct LocalInfo {
    pub name: Symbol,
    pub defined_where: Span,
    pub ty_id: NodeId,
    pub mutable: bool,
}
