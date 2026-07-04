use ayuc_id::hir::{DefId, HirId};
use ayuc_span::symbol::Symbol;

use crate::{expr::Block, ty::Ty};

#[derive(Debug)]
pub struct Item {
    pub id: DefId,
    pub hir_id: HirId,
    pub kind: ItemKind,
}

#[derive(Debug)]
pub enum ItemKind {
    Fn(FnItem),
    ExternFn(ExternFnItem),
}

#[derive(Debug)]
pub struct Parameter {
    pub hir_id: HirId,
    pub name: Symbol,
    pub ty: Ty,
}

#[derive(Debug)]
pub struct FnItem {
    pub name: Symbol,
    pub return_ty: Ty,
    pub block: Block,
    pub params: Vec<Parameter>,
}

#[derive(Debug)]
pub struct ExternFnItem {
    pub name: Symbol,
    pub params: Vec<Parameter>,
    pub return_ty: Ty,
}
