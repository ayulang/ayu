use ayuc_ast::{
    AssignStmt, Ast, BinExpr, Block, CallExpr, Expr, ExternFnItem, ExternModItem, FnItem, Ident,
    IfStmt, Item, LetStmt, Literal, LoopStmt, ModItem, Parameter, Pat, PatBinding, Path,
    ReturnStmt, Stmt, Ty, WhileStmt,
};

use crate::walkable::Walkable;

pub trait Visitor<'ast>: Sized {
    fn visit_ast(&mut self, ast: &'ast Ast) {
        ast.walk(self);
    }

    fn visit_item_list(&mut self, items: &'ast [Item]) {
        items.walk(self);
    }

    fn visit_item(&mut self, item: &'ast Item) {
        item.walk(self);
    }

    fn visit_fn_item(&mut self, fun: &'ast FnItem) {
        fun.walk(self);
    }

    fn visit_extern_fn_item(&mut self, extern_fun: &'ast ExternFnItem) {
        extern_fun.walk(self);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        block.walk(self);
    }

    fn visit_mod_item(&mut self, module: &'ast ModItem) {
        module.walk(self);
    }

    fn visit_extern_mod_item(&mut self, extern_module: &'ast ExternModItem) {
        extern_module.walk(self);
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        stmt.walk(self);
    }

    fn visit_break_stmt(&mut self) {}

    fn visit_return_stmt(&mut self, ret: &'ast ReturnStmt) {
        ret.walk(self);
    }

    fn visit_expr_stmt(&mut self, expr: &'ast Expr) {
        expr.walk(self);
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt) {
        assign_stmt.walk(self);
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ast LetStmt) {
        let_stmt.walk(self);
    }

    fn visit_while_stmt(&mut self, while_stmt: &'ast WhileStmt) {
        while_stmt.walk(self);
    }

    fn visit_loop_stmt(&mut self, loop_stmt: &'ast LoopStmt) {
        loop_stmt.walk(self);
    }

    fn visit_if_stmt(&mut self, if_stmt: &'ast IfStmt) {
        if_stmt.walk(self);
    }

    fn visit_pat(&mut self, pat: &'ast Pat) {
        pat.walk(self);
    }

    #[allow(unused)]
    fn visit_pat_binding(&mut self, binding: &'ast PatBinding) {}

    fn visit_pat_tuple(&mut self, elements: &'ast [Pat]) {
        elements.walk(self);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        expr.walk(self);
    }

    fn visit_binary_expression(&mut self, bin: &'ast BinExpr) {
        bin.walk(self);
    }

    fn visit_call_expression(&mut self, call: &'ast CallExpr) {
        call.walk(self);
    }

    fn visit_parenthesized_expression(&mut self, inner: &'ast Expr) {
        inner.walk(self);
    }

    fn visit_tuple(&mut self, elements: &'ast [Expr]) {
        elements.walk(self);
    }

    #[allow(unused)]
    fn visit_literal(&mut self, literal: &'ast Literal) {}

    #[allow(unused)]
    fn visit_path(&mut self, path: &'ast Path) {}

    #[allow(unused)]
    fn visit_identifier(&mut self, ident: &'ast Ident) {}

    fn visit_ty(&mut self, ty: &'ast Ty) {
        ty.walk(self);
    }

    fn visit_path_ty(&mut self, path: &'ast Path) {
        self.visit_path(path);
    }

    fn visit_tuple_ty(&mut self, elements: &'ast [Ty]) {
        elements.walk(self);
    }

    fn visit_parameter(&mut self, parameter: &'ast Parameter) {
        parameter.walk(self);
    }

    fn visit_parameter_identifier(&mut self, ident: &'ast Ident) {
        self.visit_identifier(ident);
    }

    fn visit_parameter_ty(&mut self, ty: &'ast Ty) {
        self.visit_ty(ty);
    }
}
