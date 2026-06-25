use crate::{Ident, expr::Expression};

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
    VarDecl(VariableDeclaration),
    Return(ReturnStatement),
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub ident: Ident,
    pub init: Expression,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub expr: Option<Expression>,
}
