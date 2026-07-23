use std::collections::HashMap;

use ayuc_ast::{Ast, Item, ItemKind, LetStmt, PatKind, Stmt, StmtKind};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_span::{Span, symbol::Symbol};

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
                        Diagnostic::error(self.file_id, signature_span, Recovery::Fatal)
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

                for stmt in &child.block.children {
                    self.bc_walk_stmt(stmt);
                }
            }
            ItemKind::ExternFn(_) => {}
        }
    }

    fn bc_walk_stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Let(let_stmt) = &stmt.kind {
            self.bc_walk_let_stmt(let_stmt)
        }
    }

    fn bc_walk_let_stmt(&mut self, let_stmt: &LetStmt) {
        let PatKind::Tuple(inner) = &let_stmt.pat.kind else {
            return;
        };

        let mut defined_identifiers: HashMap<Symbol, Vec<Span>> = HashMap::new();
        let mut queue = Vec::from_iter(inner.iter());

        while let Some(pat) = queue.pop() {
            match &pat.kind {
                PatKind::Binding(binding) => {
                    defined_identifiers
                        .entry(binding.sym)
                        .and_modify(|s| s.push(pat.span))
                        .or_insert(vec![pat.span]);
                }
                PatKind::Tuple(inner) => queue.extend(inner),
            }
        }

        for (sym, mut definitions) in defined_identifiers {
            if definitions.len() <= 1 || sym.as_str() == "_" {
                continue;
            }

            definitions.sort_by_key(|a| a.start);

            let diag = Diagnostic::error(self.file_id, let_stmt.pat.span, Recovery::Fatal)
                .with_message(format!("`{sym}` is defined more than once in pattern"))
                .with_label(Label::help(definitions.remove(0), "first definition here"));

            for def in definitions {
                self.dcx.emit(
                    diag.clone()
                        .with_label(Label::primary(def, "second definition here")),
                );
            }
        }
    }
}
