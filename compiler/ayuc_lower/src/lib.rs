use ayuc_ast as ast;
use ayuc_hir::{self as hir};

use ayuc_tyctx::TyCtx;

pub struct AstLowering<'a> {
    pub ty_ctx: &'a mut TyCtx,
    pub package: &'a mut hir::Package,
}

impl<'a> AstLowering<'a> {
    pub fn new(ty_ctx: &'a mut TyCtx, package: &'a mut hir::Package) -> Self {
        Self { ty_ctx, package }
    }

    pub fn lower(&mut self, ast: &ayuc_ast::Ast) {
        for item in &ast.items {
            let item = self.lower_item(item);

            self.package.items.push(item);
        }
    }

    fn lower_item(&mut self, item: &ast::Item) -> hir::Item {
        match item {
            ast::Item::Fn(fn_decl) => hir::Item::Fn(hir::FnItem {
                id: self.package.def_id_allocator.allocate(),
                hir_id: self.package.hir_id_allocator.allocate(),
                name: fn_decl.ident.sym,
                return_ty: hir::Ty::None,
                block: self.lower_block(&fn_decl.block),
            }),
            ast::Item::ExternFn(extern_fn) => hir::Item::ExternFn(hir::ExternFnItem {
                id: self.package.def_id_allocator.allocate(),
                hir_id: self.package.hir_id_allocator.allocate(),
                name: extern_fn.ident.sym,
            }),
        }
    }

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            stmts: block.children.iter().map(|s| self.lower_stmt(s)).collect(),
        }
    }

    fn lower_stmt(&mut self, stmt: &ast::Statement) -> hir::Statement {
        match stmt {
            ast::Statement::Expr(expr) => hir::Statement::Expr(self.lower_expr(expr)),
            ast::Statement::VarDecl(var_decl) => {
                hir::Statement::VarDecl(hir::VariableDeclaration {
                    ident: var_decl.ident.sym,
                    init: self.lower_expr(&var_decl.init),
                })
            }
        }
    }

    fn lower_expr(&mut self, expr: &ast::Expression) -> hir::Expression {
        match expr {
            ast::Expression::Identifier(ident) => hir::Expression::Ident(ident.sym),
            ast::Expression::Call(call) => hir::Expression::Call(ayuc_hir::CallExpr {
                callee: Box::new(self.lower_expr(&call.callee)),
                args: call.args.iter().map(|e| self.lower_expr(e)).collect(),
            }),
            ast::Expression::Lit(lit) => hir::Expression::Lit(match lit {
                ast::Literal::Str { span: _, data } => hir::Literal::Str(*data),
            }),
        }
    }
}
