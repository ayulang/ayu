use ayuc_id::{
    BodyId, ModuleId, TyId,
    hir::{DefId, HirId, LocalId},
};
use ayuc_span::{Span, symbol::Symbol};

#[derive(Debug)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub struct Item {
    pub def_id: DefId,
    pub hir_id: Option<HirId>,

    pub defined_at: Span,
    pub vis: Visibility,
    pub kind: ItemKind,
}

impl Item {
    /// Returns an [`Option<TyId>`], if the item's kind is a function.
    pub fn fn_ty_id(&self) -> Option<TyId> {
        match &self.kind {
            ItemKind::Fn(FnItem { ty_id, .. }) | ItemKind::ExternFn(ExternFnItem { ty_id, .. }) => {
                *ty_id
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ItemKind {
    Fn(FnItem),
    ExternFn(ExternFnItem),
    InlineMod(InlineModItem),
    ExternMod(ExternModItem),
    FileMod(FileModItem),
}

#[derive(Debug)]
pub struct FnItem {
    pub ty_id: Option<TyId>,
    pub body_id: Option<BodyId>,

    pub name: Symbol,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug)]
pub struct ExternFnItem {
    pub ty_id: Option<TyId>,

    pub name: Symbol,
    pub ffi_name: Option<Symbol>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug)]
pub struct InlineModItem {
    pub name: Symbol,
    pub items: Vec<DefId>,
}

#[derive(Debug)]
pub struct ExternModItem {
    pub name: Symbol,
    pub ffi_name: Option<Symbol>,
    pub items: Vec<DefId>,
}

#[derive(Debug)]
pub struct FileModItem {
    pub module_id: ModuleId,

    pub name: Symbol,
}

#[derive(Debug)]
pub struct Parameter {
    pub ty_id: Option<TyId>,
    pub local_id: Option<LocalId>,

    pub name: Symbol,
}
