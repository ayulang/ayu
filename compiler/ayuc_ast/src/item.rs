use crate::expr::{Block, Ident};

#[derive(Debug)]
pub enum Item {
    Fn(FnDecl),
    ExternFn(ExternFnDecl),
}

#[derive(Debug)]
pub struct FnDecl {
    pub parameters: ParameterList,
    pub ident: Ident,
    pub block: Block,
}

#[derive(Debug)]
pub struct ExternFnDecl {
    pub parameters: ParameterList,
    pub ident: Ident,
}

#[derive(Debug, Default)]
pub struct ParameterList;
