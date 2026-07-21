use ayuc_id::ast::NodeId;
use ayuc_span::{Span, symbol::Symbol};

use crate::{Path, Stmt};

#[derive(Debug)]
pub struct Expr {
    pub span: Span,
    pub id: NodeId,
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum ExprKind {
    Lit(Literal),
    Path(Path),
    Call(CallExpr),
    Binary(BinExpr),
    Parenthesized(Box<Expr>),
    Tuple(Vec<Expr>),
}

impl ExprKind {
    pub const UNIT: Self = Self::Tuple(Vec::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// a + b
    Add,
    /// a > b
    Gt,
    // a >= b
    GtOrEqual,
    // a - b
    Minus,
    // a < b
    Lt,
    // a <= b
    LtOrEqual,
    /// a == b
    EqualsEquals,
    /// a != b
    NotEquals,
    /// a * b
    Mul,
    /// a / b
    Div,
    /// a % b
    Modulus,
}

#[derive(Debug)]
pub struct BinExpr {
    pub left: Box<Expr>,
    pub operator: Operator,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub enum Literal {
    Str {
        span: Span,
        data: Symbol,
    },
    InterpolatedStr {
        span: Span,
        segments: Vec<IntlSegment>,
    },
    Integer {
        span: Span,
        value: i64,
    },
    Bool {
        value: bool,
    },
}

#[derive(Debug)]
pub enum IntlSegment {
    Text(Symbol),
    Var(Ident),
}

#[derive(Debug)]
pub struct Ident {
    pub id: NodeId,
    pub span: Span,
    pub sym: Symbol,
}

#[derive(Debug)]
pub struct Block {
    pub span: Span,
    pub children: Vec<Stmt>,
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}
