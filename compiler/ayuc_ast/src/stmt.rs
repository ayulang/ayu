use crate::{Ident, expr::Expression};

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
    VarDecl(VariableDeclaration),
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub ident: Ident,
    pub init: Expression,
}
