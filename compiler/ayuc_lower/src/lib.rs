mod scope;

use ayuc_ast::{self as ast};
use ayuc_hir::{self as hir};

use ayuc_id::{
    ast::NodeId,
    hir::{DefId, HirId},
};
use ayuc_resolve::Resolver;
use ayuc_tyctx::TyCtx;
use bimap::BiHashMap;

use crate::scope::ScopeStack;

pub struct AstLowering<'a> {
    _ty_ctx: &'a mut TyCtx,
    resolver: &'a Resolver,
    package: hir::Package,
    def_mappings: BiHashMap<NodeId, DefId>,
    id_mappings: BiHashMap<NodeId, HirId>,
    stack: ScopeStack,
}

impl<'a> AstLowering<'a> {
    pub fn new(_ty_ctx: &'a mut TyCtx, resolver: &'a Resolver) -> Self {
        let id = _ty_ctx.mint_package_id();

        Self {
            _ty_ctx,
            resolver,
            package: hir::Package::new(id),
            def_mappings: BiHashMap::new(),
            id_mappings: BiHashMap::new(),
            stack: ScopeStack::new(),
        }
    }

    #[must_use]
    pub fn lower(mut self, ast: &ayuc_ast::Ast) -> hir::Package {
        for item in &ast.items {
            let def_id = self.package.items.insert(hir::Item::dummy());

            self.def_mappings.insert(item.id, def_id);
            self.stack.register_def(
                match &item.kind {
                    ast::ItemKind::Fn(ast::FnDecl { ident, .. })
                    | ast::ItemKind::ExternFn(ast::ExternFnDecl { ident, .. }) => ident.sym,
                },
                def_id,
            );
        }

        for item in &ast.items {
            let def_id = self
                .def_mappings
                .get_by_left(&item.id)
                .copied()
                .expect("item escaped two-pass");

            self.package.items[def_id] = self.lower_item(item);
        }

        self.package
    }

    #[must_use]
    fn lower_id(&mut self, id: NodeId) -> HirId {
        if self.id_mappings.get_by_left(&id).is_some() {
            panic!("tried to lower NodeId ({id:?}) into HirId: it has already been lowered");
        }

        let hir_id = self.package.hir_id_allocator.allocate();

        self.id_mappings.insert(id, hir_id);

        hir_id
    }

    fn lower_fn_item(&mut self, fun: &ast::FnDecl) -> hir::FnItem {
        let name = fun.ident.sym;
        let params = fun
            .parameters
            .parameters
            .iter()
            .map(|p| hir::Parameter {
                hir_id: self.package.hir_id_allocator.allocate(),
                name: p.ident.sym,
                ty: self.lower_ty(&p.ty),
            })
            .collect::<Vec<_>>();

        let return_ty = self.lower_ty(&fun.return_ty);

        self.stack.enter();

        for param in &params {
            let name = param.name;
            let local_id = self
                .package
                .locals
                .insert_with_key(move |k| hir::Local { id: k, name });

            self.stack.register_local(name, local_id);
        }

        let block = self.lower_block(&fun.block);

        self.stack.leave();

        hir::FnItem {
            name,
            block,
            params,
            return_ty,
        }
    }

    fn lower_item(&mut self, item: &ast::Item) -> hir::Item {
        let id = self
            .def_mappings
            .get_by_left(&item.id)
            .copied()
            .expect("unable to find DefId in mappings");

        let hir_id = self.lower_id(item.id);

        let kind = match &item.kind {
            ast::ItemKind::Fn(fun) => hir::ItemKind::Fn(self.lower_fn_item(fun)),
            ast::ItemKind::ExternFn(extern_fun) => hir::ItemKind::ExternFn(hir::ExternFnItem {
                name: extern_fun.ident.sym,
                return_ty: self.lower_ty(&extern_fun.return_ty),
                params: extern_fun
                    .parameters
                    .parameters
                    .iter()
                    .map(|p| hir::Parameter {
                        hir_id: self.package.hir_id_allocator.allocate(),
                        name: p.ident.sym,
                        ty: self.lower_ty(&p.ty),
                    })
                    .collect(),
            }),
        };

        hir::Item { id, hir_id, kind }
    }

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            stmts: block.children.iter().map(|s| self.lower_stmt(s)).collect(),
        }
    }

    fn lower_stmt(&mut self, stmt: &ast::Stmt) -> hir::Stmt {
        let id = self.lower_id(stmt.id);
        let kind = match &stmt.kind {
            ast::StmtKind::Expr(expr) => hir::StmtKind::Expr(self.lower_expr(expr)),
            ast::StmtKind::Let(decl) => hir::StmtKind::Let(hir::LetStmt {
                ident: decl.ident.sym,
                init: self.lower_expr(&decl.init),
            }),
            ast::StmtKind::Return(ret) => hir::StmtKind::Return(hir::ReturnStmt {
                expr: ret.expr.as_ref().map(|expr| self.lower_expr(expr)),
            }),
            ayuc_ast::StmtKind::If(if_stmt) => hir::StmtKind::If(hir::IfStmt {
                expr: self.lower_expr(&if_stmt.expr),
                block: self.lower_block(&if_stmt.block),
            }),
        };

        if let hir::StmtKind::Let(decl) = &kind {
            let name = decl.ident;
            let local_id = self
                .package
                .locals
                .insert_with_key(move |k| hir::Local { id: k, name });

            self.stack.register_local(name, local_id);
        }

        hir::Stmt { id, kind }
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> hir::Expr {
        let id = self.lower_id(expr.id);
        let kind = match &expr.kind {
            ast::ExprKind::Identifier(ident) => hir::ExprKind::Ref(
                self.stack
                    .lookup(ident.sym)
                    .expect("unable to find identifier"), // TODO: make this a diagnostic instead
            ),
            ast::ExprKind::Call(call) => hir::ExprKind::Call(ayuc_hir::CallExpr {
                callee: Box::new(self.lower_expr(&call.callee)),
                args: call.args.iter().map(|e| self.lower_expr(e)).collect(),
            }),
            ast::ExprKind::Lit(lit) => hir::ExprKind::Lit(match lit {
                ast::Literal::Str { span: _, data } => hir::Literal::Str(*data),
                ast::Literal::Integer { span: _, value } => hir::Literal::Integer(*value),
            }),
            ast::ExprKind::Binary(bin) => hir::ExprKind::Binary(hir::BinExpr {
                left: Box::new(self.lower_expr(&bin.left)),
                operator: match bin.operator {
                    ast::Operator::Add => hir::BinaryOp::Add,
                    ayuc_ast::Operator::Gt => hir::BinaryOp::Gt,
                },
                right: Box::new(self.lower_expr(&bin.right)),
            }),
        };

        hir::Expr { id, kind }
    }

    fn lower_ty(&mut self, ty: &ast::Ty) -> hir::Ty {
        self.resolver.get_res(ty.id)
    }
}
