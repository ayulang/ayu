use crate::{Ident, expr::Expression};

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
    Let(LetStatement),
    Return(ReturnStatement),
}

#[derive(Debug)]
pub struct LetStatement {
    pub ident: Ident,
    pub init: Expression,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub expr: Option<Expression>,
}
