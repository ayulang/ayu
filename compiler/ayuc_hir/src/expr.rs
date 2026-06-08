use ayuc_span::symbol::Symbol;

use crate::stmt::Statement;

#[derive(Debug)]
pub enum Expression {
    Call(CallExpr),
    Lit(Literal),
    Ident(Symbol),
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
}

#[derive(Debug)]
pub enum Literal {
    Str(Symbol),
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Statement>,
}
