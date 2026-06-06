use crate::node::stmt::Statement;

pub mod leaf;
pub mod stmt;

/// A single node in an abstract syntax tree.
#[derive(Debug)]
pub enum Node {
    Statement(Statement),
}
