use ayuc_hir::{Expression, Item, Package, Statement};
use ayuc_span::symbol::Symbol;

use crate::scope::{ScopeCtx, ScopeId};

pub struct ResolveNamesPass<'a> {
    pub scope_ctx: &'a mut ScopeCtx,
}

impl<'a> ResolveNamesPass<'a> {
    pub fn new(scope_ctx: &'a mut ScopeCtx) -> Self {
        Self { scope_ctx }
    }

    pub fn run(&self, package: &Package) {
        for item in &package.items {
            if let Item::Fn(function) = item {
                let scope = self.scope_ctx.hir_scope_id(function.hir_id).unwrap(); // assume it exists, for now

                self.check_statements(scope, &function.block.stmts);
            }
        }
    }

    fn check_statements(&self, scope: ScopeId, statements: &[Statement]) {
        for statement in statements {
            match statement {
                Statement::Expr(expr) => self.check_expr(scope, expr),
            }
        }
    }

    fn check_expr(&self, scope: ScopeId, expr: &Expression) {
        match expr {
            Expression::Call(call) => {
                self.check_expr(scope, &call.callee);

                for arg in &call.args {
                    self.check_expr(scope, arg);
                }
            }
            Expression::Lit(_) => {}
            Expression::Ident(ident) => self.check_identifier(scope, *ident),
        }
    }

    fn check_identifier(&self, scope: ScopeId, ident: Symbol) {
        if self.scope_ctx.lookup(scope, ident).is_none() {
            println!("ident {:?} not found in current scope!!", ident)
        }
    }
}
