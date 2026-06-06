use ayuc_span::Span;

use crate::node::Node;

#[derive(Debug)]
pub struct Block {
    pub span: Span,
    pub children: Vec<Node>,
}
