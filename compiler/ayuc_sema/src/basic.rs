use ayuc_ast::{Ast, Item, ItemKind};
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_span::Span;

use crate::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub fn basiccheck(&mut self, ast: &Ast) {
        for item in &ast.items {
            self.bc_walk_item(item, None);
        }
    }

    fn bc_walk_item(&mut self, item: &Item, extern_block: Option<(Span, Span)>) {
        match &item.kind {
            ItemKind::ExternMod(decl) => {
                let block = Some((
                    Span::from((item.span.start, decl.block_span.start + 1)),
                    Span::from((item.span.end - 1, item.span.end)),
                ));

                for item in &decl.items {
                    self.bc_walk_item(item, block);
                }
            }
            ItemKind::InlineMod(decl) => {
                for item in &decl.items {
                    self.bc_walk_item(item, extern_block);
                }
            }
            ItemKind::Fn(child) => {
                if let Some((start, end)) = extern_block {
                    let signature_span = Span::from((item.span.start, child.return_ty.span.end));

                    self.dcx.emit(
                        Diagnostic::error(self.file_id, signature_span)
                            .with_message("non-extern item within extern module")
                            .with_label(Label::help(start, "extern module starts here"))
                            .with_label(Label::primary(
                                signature_span,
                                "not permitted within an extern module",
                            ))
                            .with_label(Label::help(end, "extern module ends here"))
                            .with_help(format!(
                                "consider moving {name} into a regular module",
                                name = child.ident.sym
                            )),
                    );
                }
            }
            ItemKind::ExternFn(_) => {}
        }
    }
}
