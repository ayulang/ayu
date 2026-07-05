use ayuc_id::hir::{DefId, LocalId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Def {
    Def(DefId),
    Local(LocalId),
    Error,
}
