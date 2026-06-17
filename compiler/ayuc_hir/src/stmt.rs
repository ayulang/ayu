use ayuc_span::symbol::Symbol;

use crate::expr::Expression;

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
    VarDecl(VariableDeclaration),
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub ident: Symbol,
    pub init: Expression,
}
