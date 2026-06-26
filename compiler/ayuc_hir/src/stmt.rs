use ayuc_id::hir::HirId;
use ayuc_span::symbol::Symbol;

use crate::Expr;

#[derive(Debug)]
pub struct Stmt {
    pub id: HirId,
    pub kind: StmtKind,
}

#[derive(Debug)]
pub enum StmtKind {
    Expr(Expr),
    VarDecl(LetStmt),
    Return(ReturnStmt),
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
