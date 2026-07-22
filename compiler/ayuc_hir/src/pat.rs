use ayuc_id::hir::HirId;
use ayuc_span::symbol::Symbol;

#[derive(Debug)]
pub struct Pat {
    pub id: HirId,
    pub kind: PatKind,
}

#[derive(Debug)]
pub enum PatKind {
    Identifier { sym: Symbol, mutable: bool },
    Tuple(Vec<Pat>),
}
