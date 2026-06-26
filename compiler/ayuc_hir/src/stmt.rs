use ayuc_span::symbol::Symbol;

use crate::expr::Expression;

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
    VarDecl(VariableDeclaration),
    Return(ReturnStatement),
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub ident: Symbol,
    pub init: Expression,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub expr: Option<Expression>,
}
