use ayuc_span::symbol::Symbol;

use crate::{DefId, expr::Block, ty::Ty};

#[derive(Debug)]
pub enum Item {
    Fn(FnItem),
}

#[derive(Debug)]
pub struct FnItem {
    pub id: DefId,
    pub name: Symbol,
    pub return_ty: Ty,
    pub block: Block,
}
