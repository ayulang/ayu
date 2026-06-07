use crate::node::Node;

pub mod node;

#[derive(Debug)]
pub struct Ast {
    pub nodes: Vec<Node>,
}
