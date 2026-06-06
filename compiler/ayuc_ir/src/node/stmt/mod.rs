use crate::node::expr::Expression;

pub mod block;

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
}
