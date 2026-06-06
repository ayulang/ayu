use crate::node::expr::call::CallExpression;

pub mod call;

#[derive(Debug)]
pub enum Expression {
    Call(CallExpression),
}
