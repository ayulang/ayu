use ayuc_id::ast::NodeId;
use ayuc_span::{Span, symbol::Symbol};

use crate::Stmt;

#[derive(Debug)]
pub struct Expr {
    pub span: Span,
    pub id: NodeId,
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum ExprKind {
    Lit(Literal),
    Identifier(Ident),
    Call(CallExpr),
    Binary(BinExpr),
}

#[derive(Debug)]
pub enum Operator {
    /// a + b
    Add,
}

#[derive(Debug)]
pub struct BinExpr {
    pub left: Box<Expr>,
    pub operator: Operator,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub enum Literal {
    Str { span: Span, data: Symbol },
    Integer { span: Span, value: i64 },
}

#[derive(Debug)]
pub struct Ident {
    pub span: Span,
    pub sym: Symbol,
}

#[derive(Debug)]
pub struct Block {
    pub span: Span,
    pub children: Vec<Stmt>,
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}
