use ayuc_id::hir::{DefId, HirId};
use ayuc_span::symbol::Symbol;

use crate::{expr::Block, ty::Ty};

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub struct Item {
    pub vis: Visibility,
    pub id: DefId,
    pub hir_id: HirId,
    pub kind: ItemKind,
}

#[derive(Debug)]
pub enum ItemKind {
    Fn(FnItem),
    ExternFn(ExternFnItem),
    InlineMod(InlineModItem),
    ExternMod(ExternModItem),
}

#[derive(Debug)]
pub struct ExternModItem {
    pub name: Symbol,
    pub ffi_name: Option<Symbol>,
    pub items: Vec<DefId>,
}

#[derive(Debug)]
pub struct InlineModItem {
    pub name: Symbol,
    pub items: Vec<DefId>,
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
    pub ffi_name: Option<Symbol>,
    pub params: Vec<Parameter>,
    pub return_ty: Ty,
}
