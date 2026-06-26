use ayuc_span::symbol::Symbol;

use crate::stmt::Statement;

#[derive(Debug)]
pub enum Expression {
    Call(CallExpr),
    Lit(Literal),
    Ident(Symbol),
    Binary(BinaryExpression),
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOp,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
}

#[derive(Debug)]
pub enum Literal {
    Str(Symbol),
    Integer(i64),
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Statement>,
}
