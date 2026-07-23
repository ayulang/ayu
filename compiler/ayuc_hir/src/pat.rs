use ayuc_id::hir::HirId;
use ayuc_span::symbol::Symbol;

#[derive(Debug)]
pub struct Pat {
    pub id: HirId,
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
