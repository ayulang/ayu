use crate::expr::{Block, Ident};

#[derive(Debug)]
pub enum Item {
    Fn(FnDecl),
}

#[derive(Debug)]
pub struct FnDecl {
    pub parameters: ParameterList,
    pub ident: Ident,
    pub block: Block,
}

#[derive(Debug, Default)]
pub struct ParameterList;
