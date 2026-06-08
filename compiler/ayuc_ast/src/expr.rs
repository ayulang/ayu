use ayuc_span::{Span, symbol::Symbol};

use crate::stmt::Statement;

#[derive(Debug)]
pub enum Expression {
    Lit(Literal),
    Identifier(Ident),
    Call(Call),
}

#[derive(Debug)]
pub enum Literal {
    Str { span: Span, data: Symbol },
}

#[derive(Debug)]
pub struct Ident {
    pub span: Span,
    pub sym: Symbol,
}

#[derive(Debug)]
pub struct Block {
    pub span: Span,
    pub children: Vec<Statement>,
}

#[derive(Debug)]
pub struct Call {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
}
