use crate::node::{leaf::ident::Ident, stmt::block::Block};

#[derive(Debug)]
pub struct FnDecl {
    pub parameters: ParameterList,
    pub ident: Ident,
    pub block: Block,
}

#[derive(Debug, Default)]
pub struct ParameterList;
