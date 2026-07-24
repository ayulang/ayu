//! This module contains the definition of the [`Visitor`] trait used for traversing an [`Ast`].

use ayuc_ast::{
    AssignStmt, Ast, BinExpr, Block, CallExpr, Expr, ExternFnItem, ExternModItem, FnItem, Ident,
    IfStmt, Item, LetStmt, Literal, LoopStmt, ModItem, Parameter, Pat, PatBinding, Path,
    ReturnStmt, Stmt, Ty, WhileStmt,
};

use crate::walkable::Walkable;

/// Visitor trait for traversing the AST using the Visitor pattern.
///
/// Default implementations recursively walk the AST via [`Walkable`] implementations. Override
///   methods to add custom logic at specific nodes. All methods take immutable references, so
///   this is for read-only traversal.
#[allow(unused_variables)]
pub trait Visitor<'ast>: Sized {
    /// Entry point that visits the entire AST.
    fn visit_ast(&mut self, ast: &'ast Ast) {
        ast.walk(self);
    }

    /// Visits a list of items in order.
    fn visit_item_list(&mut self, items: &'ast [Item]) {
        items.walk(self);
    }

    /// Visits a single item (function, module, ...).
    fn visit_item(&mut self, item: &'ast Item) {
        item.walk(self);
    }

    /// Visits a function item.
    fn visit_fn_item(&mut self, fun: &'ast FnItem) {
        fun.walk(self);
    }

    /// Visits an extern function item.
    fn visit_extern_fn_item(&mut self, extern_fun: &'ast ExternFnItem) {
        extern_fun.walk(self);
    }

    /// Visits a block expression.
    fn visit_block_expr(&mut self, block: &'ast Block) {
        block.walk(self);
    }

    /// Visits a module item.
    fn visit_mod_item(&mut self, module: &'ast ModItem) {
        module.walk(self);
    }

    /// Visits an extern module item.
    fn visit_extern_mod_item(&mut self, extern_module: &'ast ExternModItem) {
        extern_module.walk(self);
    }

    /// Visits a statement (break, if, let, ...).
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        stmt.walk(self);
    }

    /// Visits a break statement.
    ///
    /// Unlike other functions within this trait, this function takes no additional parameter. This
    ///   is because a break statement doesn't have any children.
    fn visit_break_stmt(&mut self) {}

    /// Visits a return statement.
    fn visit_return_stmt(&mut self, ret: &'ast ReturnStmt) {
        ret.walk(self);
    }

    /// Visits an expression statement.
    fn visit_expr_stmt(&mut self, expr: &'ast Expr) {
        self.visit_expr(expr);
    }

    /// Visits an assignment statement.
    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt) {
        assign_stmt.walk(self);
    }

    /// Visits a let statement.
    fn visit_let_stmt(&mut self, let_stmt: &'ast LetStmt) {
        let_stmt.walk(self);
    }

    /// Visits a while statement.
    fn visit_while_stmt(&mut self, while_stmt: &'ast WhileStmt) {
        while_stmt.walk(self);
    }

    /// Visits a loop statement.
    fn visit_loop_stmt(&mut self, loop_stmt: &'ast LoopStmt) {
        loop_stmt.walk(self);
    }

    /// Visits an if statement.
    fn visit_if_stmt(&mut self, if_stmt: &'ast IfStmt) {
        if_stmt.walk(self);
    }

    /// Visits a pattern that is used within a let statement.
    fn visit_pat(&mut self, pat: &'ast Pat) {
        pat.walk(self);
    }

    /// Visits a pattern that binds to a name.
    ///
    /// This node is a leaf node and has no further traversal logic.
    fn visit_pat_binding(&mut self, binding: &'ast PatBinding) {}

    /// Visits a tuple pattern.
    fn visit_pat_tuple(&mut self, elements: &'ast [Pat]) {
        elements.walk(self);
    }

    /// Visits an expression.
    fn visit_expr(&mut self, expr: &'ast Expr) {
        expr.walk(self);
    }

    /// Visits a binary expression.
    fn visit_binary_expression(&mut self, bin: &'ast BinExpr) {
        bin.walk(self);
    }

    /// Visits a call expression.
    fn visit_call_expression(&mut self, call: &'ast CallExpr) {
        call.walk(self);
    }

    /// Visits a parenthesized expression.
    fn visit_parenthesized_expression(&mut self, inner: &'ast Expr) {
        inner.walk(self);
    }

    /// Visits a tuple expression.
    fn visit_tuple_expr(&mut self, elements: &'ast [Expr]) {
        elements.walk(self);
    }

    /// Visits a literal (string, boolean, ...).
    ///
    /// This node is a leaf node and has no further traversal logic.
    fn visit_literal(&mut self, literal: &'ast Literal) {}

    /// Visits a path.
    ///
    /// This node is a leaf node and has no further traversal logic.
    fn visit_path(&mut self, path: &'ast Path) {}

    /// Visits an identifier.
    ///
    /// This node is a leaf node and has no further traversal logic.
    fn visit_identifier(&mut self, ident: &'ast Ident) {}

    /// Visits a type.
    fn visit_ty(&mut self, ty: &'ast Ty) {
        ty.walk(self);
    }

    /// Visits a type that is a path.
    ///
    /// This method is basically a proxy for calling [`Self::visit_path`]. It exists to give you the
    ///   option whether to let path types bleed into [`Self::visit_path`].
    fn visit_path_ty(&mut self, path: &'ast Path) {
        self.visit_path(path);
    }

    /// Visits a tuple type.
    fn visit_tuple_ty(&mut self, elements: &'ast [Ty]) {
        elements.walk(self);
    }

    /// Visits a parameter.
    fn visit_parameter(&mut self, parameter: &'ast Parameter) {
        parameter.walk(self);
    }

    /// Visits a parameter's identifier.
    ///
    /// This method is basically a proxy for calling [`Self::visit_identifier`]. It exists to give you the
    ///   option whether to let parameter identifiers bleed into [`Self::visit_identifier`].
    fn visit_parameter_identifier(&mut self, ident: &'ast Ident) {
        self.visit_identifier(ident);
    }

    /// Visits a parameter's type.
    ///
    /// This method is basically a proxy for calling [`Self::visit_ty`]. It exists to give you the
    ///   option whether to let parameter types bleed into [`Self::visit_ty`].
    fn visit_parameter_ty(&mut self, ty: &'ast Ty) {
        self.visit_ty(ty);
    }
}
