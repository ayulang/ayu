use crate::node::decl::function::FnDecl;

pub mod function;

#[derive(Debug)]
pub enum Declaration {
    Function(FnDecl),
}
