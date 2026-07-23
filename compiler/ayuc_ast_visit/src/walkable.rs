use ayuc_ast::{
    AssignStmt, Ast, BinExpr, Block, CallExpr, Expr, ExternFnItem, ExternModItem, FnItem, IfStmt,
    Item, LetStmt, LoopStmt, ModItem, Parameter, Pat, ReturnStmt, Stmt, Ty, WhileStmt,
};

use crate::{visitor::Visitor, walk};

macro_rules! make_walkable {
    ($node:ident => $proxy:path) => {
        impl<'ast> Walkable<'ast> for $node {
            #[inline(always)]
            fn walk<V: Visitor<'ast>>(&'ast self, visitor: &mut V) {
                $proxy(visitor, self)
            }
        }
    };

    ([$node:ident] => $method:ident) => {
        impl<'ast> Walkable<'ast> for [$node] {
            #[inline(always)]
            fn walk<V: Visitor<'ast>>(&'ast self, visitor: &mut V) {
                for inner in self {
                    visitor.$method(inner);
                }
            }
        }
    };
}

/// Walkable trait providing traversal logic for AST nodes. It's used to decide what visitor
///   functions are called when and where in the current node.
pub trait Walkable<'ast> {
    /// Walks the node and calls visitor functions for traversal.
    fn walk<V: Visitor<'ast>>(&'ast self, visitor: &mut V);
}

make_walkable!(Ast => walk::walk_ast);

make_walkable!(Item => walk::walk_item);
make_walkable!([Item] => visit_item);

make_walkable!(ModItem => walk::walk_mod_item);
make_walkable!(ExternModItem => walk::walk_extern_mod_item);
make_walkable!(FnItem => walk::walk_fn_item);
make_walkable!(ExternFnItem => walk::walk_extern_fn_item);

make_walkable!(Stmt => walk::walk_stmt);
make_walkable!([Stmt] => visit_stmt);
make_walkable!(ReturnStmt => walk::walk_return_stmt);
make_walkable!(AssignStmt => walk::walk_assign_stmt);
make_walkable!(WhileStmt => walk::walk_while_stmt);
make_walkable!(IfStmt => walk::walk_if_stmt);
make_walkable!(LoopStmt => walk::walk_loop_stmt);
make_walkable!(LetStmt => walk::walk_let_stmt);

make_walkable!(Expr => walk::walk_expr);
make_walkable!([Expr] => visit_expr);

make_walkable!(Block => walk::walk_block);
make_walkable!(BinExpr => walk::walk_binary_expression);
make_walkable!(CallExpr => walk::walk_call_expression);

make_walkable!(Pat => walk::walk_pat);
make_walkable!([Pat] => visit_pat);

make_walkable!(Ty => walk::walk_ty);
make_walkable!([Ty] => visit_ty);

make_walkable!(Parameter => walk::walk_parameter);
make_walkable!([Parameter] => visit_parameter);
