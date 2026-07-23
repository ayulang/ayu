use std::collections::HashMap;

use ayuc_ast::{ExternModItem, FnItem, Item, LetStmt, PatKind};
use ayuc_ast_visit::{visitor::Visitor, walkable::Walkable};
use ayuc_diagnostic::{Diagnostic, DiagnosticContext, Label, Recovery};
use ayuc_span::{Span, symbol::Symbol};

pub struct GeneralPhase<'dcx> {
    dcx: &'dcx mut DiagnosticContext,
    file_id: usize,

    extern_mod: Option<(Span, Span)>,
    item_span: Option<Span>,
}

impl<'dcx> GeneralPhase<'dcx> {
    pub fn new(dcx: &'dcx mut DiagnosticContext, file_id: usize) -> Self {
        Self {
            dcx,
            file_id,
            extern_mod: None,
            item_span: None,
        }
    }
}

impl Visitor<'_> for GeneralPhase<'_> {
    fn visit_item(&mut self, item: &'_ Item) {
        let old_span = self.item_span.replace(item.span);

        item.walk(self);

        self.item_span = old_span;
    }

    fn visit_extern_mod_item(&mut self, extern_module: &'_ ExternModItem) {
        let item_span = self.item_span.unwrap();
        let old_mod = self.extern_mod.replace((
            Span::from((item_span.start, extern_module.block_span.start + 1)),
            Span::from((item_span.end - 1, item_span.end)),
        ));

        extern_module.walk(self);

        self.extern_mod = old_mod;
    }

    fn visit_fn_item(&mut self, fun: &'_ FnItem) {
        if let Some((start, end)) = self.extern_mod {
            let item_span = self.item_span.unwrap();
            let signature_span = Span::from((item_span.start, fun.return_ty.span.end));

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
                        name = fun.ident.sym
                    )),
            );
        }

        fun.walk(self);
    }

    fn visit_let_stmt(&mut self, let_stmt: &'_ LetStmt) {
        let PatKind::Tuple(inner) = &let_stmt.pat.kind else {
            return let_stmt.walk(self);
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
