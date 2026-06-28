use ayuc_id::hir::HirId;
use ayuc_span::symbol::Symbol;

use crate::{Block, Expr};

#[derive(Debug)]
pub struct Stmt {
    pub id: HirId,
    pub kind: StmtKind,
}

#[derive(Debug)]
pub enum StmtKind {
    Expr(Expr),
    Let(LetStmt),
    Return(ReturnStmt),
    If(IfStmt),
}

#[derive(Debug)]
pub struct LetStmt {
    pub ident: Symbol,
    pub init: Expr,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct IfStmt {
    pub expr: Expr,
    pub block: Block,
}
