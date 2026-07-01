use ayuc_id::hir::HirId;
use ayuc_span::symbol::Symbol;

use crate::{Def, stmt::Stmt};

#[derive(Debug)]
pub struct Expr {
    pub id: HirId,
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum ExprKind {
    Call(CallExpr),
    Lit(Literal),
    Ref(Def),
    Binary(BinExpr),
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Minus,
    Gt,
    GtOrEqual,
    Lt,
    LtOrEqual,
    EqualsEquals,
    NotEquals,
}

#[derive(Debug)]
pub struct BinExpr {
    pub left: Box<Expr>,
    pub operator: BinaryOp,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub enum Literal {
    Str(Symbol),
    Integer(i64),
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}
