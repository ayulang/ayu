use ayuc_id::hir::{DefId, LocalId};

#[derive(Debug, Clone, Copy)]
pub enum Def {
    Local(LocalId),
    Def(DefId),
}
