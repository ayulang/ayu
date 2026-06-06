use crate::node::stmt::fn_decl::FnDecl;

pub mod block;
pub mod fn_decl;

#[derive(Debug)]
pub enum Statement {
    FnDecl(FnDecl),
}
