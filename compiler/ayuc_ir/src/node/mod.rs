use crate::node::{decl::Declaration, stmt::Statement};

pub mod decl;
pub mod leaf;
pub mod stmt;

/// A single node in an abstract syntax tree.
#[derive(Debug)]
pub enum Node {
    Stmt(Statement),
    Decl(Declaration),
}
