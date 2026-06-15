use ayuc_span::symbol::Symbol;

use crate::{DefId, HirId, expr::Block, ty::Ty};

#[derive(Debug)]
pub enum Item {
    Fn(FnItem),
    ExternFn(ExternFnItem),
}

#[derive(Debug)]
pub struct FnItem {
    pub id: DefId,
    pub hir_id: HirId,
    pub name: Symbol,
    pub return_ty: Ty,
    pub block: Block,
}

#[derive(Debug)]
pub struct ExternFnItem {
    pub id: DefId,
    pub hir_id: HirId,
    pub name: Symbol,
}
