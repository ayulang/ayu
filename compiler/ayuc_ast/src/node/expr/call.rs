use crate::node::leaf::ident::Ident;

#[derive(Debug)]
pub struct CallExpression {
    pub callee: Ident, // TODO: Add support for stuff like: x.y
    pub arguments: ArgumentList,
}

#[derive(Debug)]
pub struct ArgumentList();
