use crate::expr::Expression;

#[derive(Debug)]
pub enum Statement {
    Expr(Expression),
}
