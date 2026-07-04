use crate::Resolver;

use ayuc_ast as ast;
use ayuc_hir as hir;

impl Resolver<'_> {
    pub(crate) fn resolve_types(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.tr_walk_item(item);
        }
    }

    fn tr_walk_item(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::Fn(fun) => {
                for param in &fun.parameters.parameters {
                    self.tr_resolve_ty(&param.ty);
                }

                self.tr_resolve_ty(&fun.return_ty);
            }
            ast::ItemKind::ExternFn(extern_fun) => {
                for param in &extern_fun.parameters.parameters {
                    self.tr_resolve_ty(&param.ty);
                }

                self.tr_resolve_ty(&extern_fun.return_ty);
            }
        }
    }

    fn tr_resolve_ty(&mut self, ty: &ast::Ty) {
        let resolved = match &ty.kind {
            ast::TyKind::Unit => hir::Ty::Unit,
            ast::TyKind::Path(p) => {
                if p.segments.len() == 1 {
                    let segment = &p.segments[0];

                    if let Some(prim) = hir::PrimTy::from_name(segment.ident.sym) {
                        hir::Ty::Primitive(prim)
                    } else {
                        hir::Ty::Error
                    }
                } else {
                    hir::Ty::Error // not yet.
                }
            }
        };

        self.ty_resolutions.insert(ty.id, resolved);
    }
}
