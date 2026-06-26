use ayuc_id::hir::{DefId, HirId};
use ayuc_span::symbol::Symbol;

use crate::{expr::Block, ty::Ty};

#[derive(Debug)]
pub enum Item {
    Fn(FnItem),
    ExternFn(ExternFnItem),
}

#[derive(Debug)]
pub struct Parameter {
    pub name: Symbol,
}

#[derive(Debug)]
pub struct FnItem {
    pub id: DefId,
    pub hir_id: HirId,
    pub name: Symbol,
    pub return_ty: Ty,
    pub block: Block,
    pub params: Vec<Parameter>,
}

#[derive(Debug)]
pub struct ExternFnItem {
    pub id: DefId,
    pub hir_id: HirId,
    pub name: Symbol,
}
