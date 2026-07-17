use ayuc_id::ast::NodeId;
use ayuc_span::Span;

use crate::{
    Ty,
    expr::{Block, Ident},
};

#[derive(Debug)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub struct Item {
    pub vis: Visibility,
    pub id: NodeId,
    pub span: Span,
    pub kind: ItemKind,
}

#[derive(Debug)]
pub enum ItemKind {
    Fn(FnDecl),
    ExternFn(ExternFnDecl),
    InlineMod(InlineModItem),
    ExternMod(ExternModItem),
}

#[derive(Debug)]
pub struct ExternModItem {
    pub ident: Ident,
    pub ffi_name: Option<Ident>,
    pub block_span: Span,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct InlineModItem {
    pub ident: Ident,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct FnDecl {
    pub parameters: ParameterList,
    pub ident: Ident,
    pub block: Block,
    pub return_ty: Ty,
}

#[derive(Debug)]
pub struct ExternFnDecl {
    pub parameters: ParameterList,
    pub ffi_name: Option<Ident>,
    pub name: Ident,
    pub return_ty: Ty,
}

#[derive(Debug, Default)]
pub struct ParameterList {
    pub span: Span,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug)]
pub struct Parameter {
    pub span: Span,
    pub id: NodeId,
    pub ident: Ident,
    pub ty: Ty,
}
