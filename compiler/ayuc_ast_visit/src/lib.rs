//! This crate provides a [`Visitor`](visitor::Visitor) trait for visiting nodes
//!   in an [`Ast`](ayuc_ast::Ast), private logic for the traversal and public
//!   proxies for the private logic.

pub mod visitor;
pub(crate) mod walk;
pub mod walkable;
