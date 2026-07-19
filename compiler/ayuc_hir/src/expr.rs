use ayuc_id::hir::HirId;
use ayuc_span::symbol::Symbol;

use crate::{Def, stmt::Stmt};

#[derive(Debug)]
pub struct Expr {
    pub id: HirId,
    pub kind: ExprKind,
}

#[derive(Debug)]
pub struct Path {
    /// The target of the path. Basically a shorthand to `segments.last()`
    pub target: Def,

    /// Every single segment of the path, resolved to its definition.
    pub segments: Vec<Def>,
}

#[derive(Debug)]
pub enum ExprKind {
    Call(CallExpr),
    Lit(Literal),
    /// A path in the form of `foo::bar`
    Path(Path),
    Binary(BinExpr),
    Parenthesized(Box<Expr>),
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Minus,
    Gt,
    GtOrEqual,
    Lt,
    LtOrEqual,
    EqualsEquals,
    NotEquals,
    Div,
    Mul,
    Modulus,
}

#[derive(Debug)]
pub struct BinExpr {
    pub left: Box<Expr>,
    pub operator: BinaryOp,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub enum Literal {
    Str(Symbol),
    InterpolatedStr(Vec<IntlSegment>),
    Integer(i64),
    Bool(bool),
}

#[derive(Debug)]
pub enum IntlSegment {
    Text(Symbol),
    Var(Def),
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}
