use ayuc_id::hir::HirId;
use ayuc_span::symbol::Symbol;

use crate::{Block, Def, Expr, Ty};

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
    Assign(AssignStmt),
    Break,
    Loop(LoopStmt),
    While(WhileStmt),
}

#[derive(Debug)]
pub struct WhileStmt {
    pub expr: Expr,
    pub block: Block,
}

#[derive(Debug)]
pub struct LoopStmt {
    pub block: Block,
}

#[derive(Debug)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
}

#[derive(Debug)]
pub struct AssignStmt {
    pub ident: Def,
    pub op: AssignOp,
    pub value: Expr,
}

#[derive(Debug)]
pub struct LetStmt {
    pub ident: Symbol,
    pub ty: Ty,
    pub mutable: bool,
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
