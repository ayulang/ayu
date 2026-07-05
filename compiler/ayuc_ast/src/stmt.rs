use ayuc_id::ast::NodeId;
use ayuc_span::Span;

use crate::{Block, Expr, Ident, Ty};

#[derive(Debug)]
pub struct Stmt {
    pub id: NodeId,
    pub span: Span,
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
    pub ident: Ident,
    pub init: Expr,
    pub ty: Ty,
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
