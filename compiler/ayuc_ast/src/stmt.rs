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
    Assignment(AssignStmt),
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
pub enum AssignOperator {
    // a = b
    Assign,
    // a += b
    Add,
    // a -= b
    Subtract,
}

#[derive(Debug)]
pub struct AssignStmt {
    pub ident: Ident,
    pub operator: AssignOperator,
    pub value: Expr,
}

#[derive(Debug)]
pub struct LetStmt {
    pub ident: Ident,
    pub mutable: bool,
    pub init: Expr,
    pub ty: Ty,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub enum AlternateBranch {
    Another(Box<IfStmt>),
    Final(Block),
}

#[derive(Debug)]
pub struct IfStmt {
    pub expr: Expr,
    pub block: Block,
    pub alternate: Option<AlternateBranch>,
}
