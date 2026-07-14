use ayuc_id::hir::LocalId;
use ayuc_span::symbol::Symbol;

#[derive(Debug)]
pub struct Local {
    pub id: LocalId,
    pub name: Symbol,
    pub mutable: bool,
}
