use ayuc_id::ast::NodeId;
use ayuc_span::Span;

use crate::expr::{Block, Ident};

#[derive(Debug)]
pub struct Item {
    pub id: NodeId,
    pub span: Span,
    pub kind: ItemKind,
}

#[derive(Debug)]
pub enum ItemKind {
    Fn(FnDecl),
    ExternFn(ExternFnDecl),
}

#[derive(Debug)]
pub struct FnDecl {
    pub parameters: ParameterList,
    pub ident: Ident,
    pub block: Block,
}

#[derive(Debug)]
pub struct ExternFnDecl {
    pub parameters: ParameterList,
    pub ident: Ident,
}

#[derive(Debug, Default)]
pub struct ParameterList;
