use ayuc_id::ast::NodeId;
use ayuc_span::Span;

use crate::{Expr, Ident};

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
}

#[derive(Debug)]
pub struct LetStmt {
    pub ident: Ident,
    pub init: Expr,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
}
