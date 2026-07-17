use ayuc_id::hir::{DefId, LocalId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Def {
    Def(DefId),
    Local(LocalId),
    /// A definition that couldn't be resolved. Often comes with a diagnostic **already emitted**.
    Error,
}
