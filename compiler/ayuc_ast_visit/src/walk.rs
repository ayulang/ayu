//! This module contains private logic for traversing nodes. It's kept private to not pollute the environment. Instead,
//!   developers have to use the [`Walkable`](crate::walkable::Walkable) trait to access it, which is way more readable.

use crate::visitor::Visitor;

use ayuc_ast::Ast;

pub use expr::*;
pub use item::*;
pub use parameter::*;
pub use pat::*;
pub use stmt::*;
pub use ty::*;

/// Traverses an AST by calling [`Visitor::visit_item_list`] on the AST's item list.
pub fn walk_ast<'ast, V: Visitor<'ast>>(visitor: &mut V, ast: &'ast Ast) {
    visitor.visit_item_list(&ast.items);
}

/// Contains the logic for traversing items.
mod item {
    use ayuc_ast::{ExternFnItem, ExternModItem, FnItem, Item, ItemKind, ModItem};

    use crate::{visitor::Visitor, walkable::Walkable};

    pub fn walk_item<'ast, V: Visitor<'ast>>(visitor: &mut V, item: &'ast Item) {
        match &item.kind {
            ItemKind::ExternFn(extern_fun) => visitor.visit_extern_fn_item(extern_fun),
            ItemKind::ExternMod(extern_module) => visitor.visit_extern_mod_item(extern_module),
            ItemKind::Fn(fun) => visitor.visit_fn_item(fun),
            ItemKind::InlineMod(module) => visitor.visit_mod_item(module),
        }
    }

    pub fn walk_fn_item<'ast, V: Visitor<'ast>>(visitor: &mut V, fun: &'ast FnItem) {
        let FnItem {
            parameters,
            ident,
            block,
            return_ty,
        } = fun;

        visitor.visit_item_identifier(ident);

        parameters.parameters.walk(visitor);

        visitor.visit_ty(return_ty);
        visitor.visit_block_expr(block);
    }

    pub fn walk_extern_fn_item<'ast, V: Visitor<'ast>>(
        visitor: &mut V,
        extern_fun: &'ast ExternFnItem,
    ) {
        let ExternFnItem {
            parameters,
            ffi_name,
            name,
            return_ty,
        } = extern_fun;

        if let Some(ffi_name) = ffi_name {
            visitor.visit_item_identifier(ffi_name);
        }

        visitor.visit_item_identifier(name);

        parameters.parameters.walk(visitor);

        visitor.visit_ty(return_ty);
    }

    pub fn walk_mod_item<'ast, V: Visitor<'ast>>(visitor: &mut V, module: &'ast ModItem) {
        let ModItem { ident, items } = module;

        visitor.visit_item_identifier(ident);
        visitor.visit_item_list(items);
    }

    pub fn walk_extern_mod_item<'ast, V: Visitor<'ast>>(
        visitor: &mut V,
        extern_module: &'ast ExternModItem,
    ) {
        let ExternModItem {
            ident,
            ffi_name,
            block_span: _,
            items,
        } = extern_module;

        if let Some(ffi_name) = ffi_name {
            visitor.visit_item_identifier(ffi_name);
        }

        visitor.visit_item_identifier(ident);
        visitor.visit_item_list(items);
    }
}

/// Contains logic for traversing statements.
mod stmt {
    use ayuc_ast::{
        AlternateBranch, AssignStmt, IfStmt, LetStmt, LoopStmt, ReturnStmt, Stmt, StmtKind,
        WhileStmt,
    };

    use crate::visitor::Visitor;

    pub fn walk_stmt<'ast, V: Visitor<'ast>>(visitor: &mut V, stmt: &'ast Stmt) {
        let Stmt {
            id: _,
            span: _,
            kind,
        } = stmt;

        match kind {
            StmtKind::Break => visitor.visit_break_stmt(),
            StmtKind::Return(ret) => visitor.visit_return_stmt(ret),
            StmtKind::Expr(expr) => visitor.visit_expr_stmt(expr),
            StmtKind::Assignment(assign_stmt) => visitor.visit_assign_stmt(assign_stmt),
            StmtKind::If(if_stmt) => visitor.visit_if_stmt(if_stmt),
            StmtKind::Loop(loop_stmt) => visitor.visit_loop_stmt(loop_stmt),
            StmtKind::While(while_stmt) => visitor.visit_while_stmt(while_stmt),
            StmtKind::Let(let_stmt) => visitor.visit_let_stmt(let_stmt),
        }
    }

    pub fn walk_return_stmt<'ast, V: Visitor<'ast>>(
        visitor: &mut V,
        return_stmt: &'ast ReturnStmt,
    ) {
        let ReturnStmt { expr } = return_stmt;

        visitor.visit_expr(expr);
    }

    pub fn walk_assign_stmt<'ast, V: Visitor<'ast>>(
        visitor: &mut V,
        assign_stmt: &'ast AssignStmt,
    ) {
        let AssignStmt {
            ident,
            operator: _,
            value,
        } = assign_stmt;

        visitor.visit_identifier(ident);
        visitor.visit_expr(value);
    }

    pub fn walk_if_stmt<'ast, V: Visitor<'ast>>(visitor: &mut V, if_stmt: &'ast IfStmt) {
        let IfStmt {
            expr,
            block,
            alternate,
        } = if_stmt;

        visitor.visit_expr(expr);
        visitor.visit_block_expr(block);

        match alternate {
            Some(AlternateBranch::Another(if_stmt)) => visitor.visit_if_stmt(if_stmt),
            Some(AlternateBranch::Final(block)) => visitor.visit_block_expr(block),
            None => {}
        }
    }

    pub fn walk_loop_stmt<'ast, V: Visitor<'ast>>(visitor: &mut V, loop_stmt: &'ast LoopStmt) {
        let LoopStmt { block } = loop_stmt;

        visitor.visit_block_expr(block);
    }

    pub fn walk_while_stmt<'ast, V: Visitor<'ast>>(visitor: &mut V, while_stmt: &'ast WhileStmt) {
        let WhileStmt { expr, block } = while_stmt;

        visitor.visit_expr(expr);
        visitor.visit_block_expr(block);
    }

    pub fn walk_let_stmt<'ast, V: Visitor<'ast>>(visitor: &mut V, let_stmt: &'ast LetStmt) {
        let LetStmt { pat, init, ty } = let_stmt;

        visitor.visit_pat(pat);

        if let Some(ty) = ty {
            visitor.visit_ty(ty);
        }

        visitor.visit_expr(init);
    }
}

/// Contains logic for traversing expressions.
mod expr {
    use ayuc_ast::{BinExpr, Block, CallExpr, Expr, ExprKind};

    use crate::{visitor::Visitor, walkable::Walkable};

    pub fn walk_expr<'ast, V: Visitor<'ast>>(visitor: &mut V, expr: &'ast Expr) {
        match &expr.kind {
            ExprKind::Binary(bin_expr) => visitor.visit_binary_expression(bin_expr),
            ExprKind::Call(call_expr) => visitor.visit_call_expression(call_expr),
            ExprKind::Parenthesized(inner) => visitor.visit_parenthesized_expression(inner),
            ExprKind::Lit(literal) => visitor.visit_literal(literal),
            ExprKind::Path(path) => visitor.visit_path(path),
            ExprKind::Tuple(elements) => visitor.visit_tuple_expr(elements),
        }
    }

    pub fn walk_block<'ast, V: Visitor<'ast>>(visitor: &mut V, block: &'ast Block) {
        let Block { span: _, children } = block;

        children.walk(visitor);
    }

    pub fn walk_binary_expression<'ast, V: Visitor<'ast>>(visitor: &mut V, binary: &'ast BinExpr) {
        let BinExpr {
            left,
            operator: _,
            right,
        } = binary;

        visitor.visit_expr(left);
        visitor.visit_expr(right);
    }

    pub fn walk_call_expression<'ast, V: Visitor<'ast>>(visitor: &mut V, call: &'ast CallExpr) {
        let CallExpr { callee, args } = call;

        visitor.visit_expr(callee);

        args.walk(visitor);
    }
}

/// Contains logic for traversing patterns.
mod pat {
    use ayuc_ast::{Pat, PatKind};

    use crate::visitor::Visitor;

    pub fn walk_pat<'ast, V: Visitor<'ast>>(visitor: &mut V, pat: &'ast Pat) {
        let Pat {
            span: _,
            id: _,
            kind,
        } = pat;

        match kind {
            PatKind::Binding(binding) => visitor.visit_pat_binding(binding),
            PatKind::Tuple(elements) => visitor.visit_pat_tuple(elements),
        }
    }
}

/// Contains logic for traversing types.
mod ty {
    use ayuc_ast::{Ty, TyKind};

    use crate::visitor::Visitor;

    pub fn walk_ty<'ast, V: Visitor<'ast>>(visitor: &mut V, ty: &'ast Ty) {
        let Ty {
            id: _,
            span: _,
            kind,
        } = ty;

        match kind {
            TyKind::Path(path) => visitor.visit_path_ty(path),
            TyKind::Tuple(elements) => visitor.visit_tuple_ty(elements),
        }
    }
}

/// Contains logic for traversing parameters.
mod parameter {
    use ayuc_ast::Parameter;

    use crate::visitor::Visitor;

    pub fn walk_parameter<'ast, V: Visitor<'ast>>(visitor: &mut V, parameter: &'ast Parameter) {
        let Parameter {
            span: _,
            id: _,
            ident,
            ty,
        } = parameter;

        visitor.visit_parameter_identifier(ident);
        visitor.visit_parameter_ty(ty);
    }
}
