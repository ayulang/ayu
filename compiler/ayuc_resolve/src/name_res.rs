use ayuc_ast::{
    Ast, Block, ExternFnItem, ExternModItem, FnItem, Ident, IntlSegment, Item, ItemKind, Literal,
    ModItem, Parameter, Pat, PatKind, Path, PathSegment, Visibility,
};
use ayuc_ast_visit::{visitor::Visitor, walkable::Walkable};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_id::{
    ast::NodeId,
    hir::{DefId, LocalId},
};
use ayuc_session::{self as session, local::LocalInfo};
use ayuc_span::{Span, symbol::Symbol};

use crate::{def::Def, resolver::Resolver};

fn ident_of_item(item: &Item) -> &Ident {
    match &item.kind {
        ItemKind::InlineMod(decl) => &decl.ident,
        ItemKind::ExternMod(decl) => &decl.ident,
        ItemKind::Fn(decl) => &decl.ident,
        ItemKind::ExternFn(decl) => &decl.name,
    }
}

impl Resolver<'_, '_> {
    pub(crate) fn run_name_resolution(&mut self, ast: &Ast) {
        FirstPass { res: self }.visit_ast(ast);

        if self.dcx.requires_abort() {
            return;
        }

        SecondPass {
            res: self,
            current_item: None,
        }
        .visit_ast(ast);
    }

    fn register_def(&mut self, sym: Symbol, def_id: DefId, node_id: NodeId) {
        self.stack.register_def(sym, def_id);
        self.rcx.defs_by_node.insert(node_id, def_id);
    }

    fn register_local(&mut self, sym: Symbol, local_id: LocalId, node_id: NodeId) {
        self.stack.register_local(sym, local_id);
        self.rcx.locals_by_node.insert(node_id, local_id);
    }
}

struct FirstPass<'a, 'dcx, 'sess> {
    res: &'a mut Resolver<'dcx, 'sess>,
}

// Visitor trait is not needed for this because it is simple and custom logic.
impl FirstPass<'_, '_, '_> {
    pub fn visit_ast(&mut self, ast: &Ast) {
        for item in &ast.items {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &Item) -> Option<DefId> {
        let ident = ident_of_item(item);
        let sym = ident.sym;

        if let Some(def) = self.res.stack.current().lookup(sym) {
            let mut diag = Diagnostic::error(self.res.file_id, ident.span, Recovery::Fatal)
                .with_message(format!("the name `{}` is defined multiple times", sym));

            if let Def::Def(id) = def {
                let item = self.res.sess.item(id);

                diag = diag.with_label(Label::help(
                    match &item.kind {
                        session::ItemKind::ExternFn { signature_span, .. }
                        | session::ItemKind::Fn { signature_span, .. }
                        | session::ItemKind::InlineMod { signature_span, .. }
                        | session::ItemKind::ExternMod { signature_span, .. } => *signature_span,
                    },
                    "first definition here",
                ))
            }

            diag = diag.with_label(Label::primary(ident.span, "name is already defined"));

            self.res.dcx.emit(diag);

            return None;
        }

        let signature_span = match &item.kind {
            ItemKind::InlineMod(decl) => Span::from((item.span.start, decl.ident.span.end)),
            ItemKind::ExternMod(decl) => Span::from((item.span.start, decl.ident.span.end)),
            ItemKind::Fn(decl) => Span::from((item.span.start, decl.return_ty.span.end)),
            ItemKind::ExternFn(decl) => Span::from((item.span.start, decl.return_ty.span.end)),
        };

        let kind = match &item.kind {
            ItemKind::Fn(_decl) => session::ItemKind::Fn { signature_span },
            ItemKind::ExternFn(decl) => session::ItemKind::ExternFn {
                ffi_name: decl.ffi_name.as_ref().map(|i| i.sym),
                signature_span,
            },
            ItemKind::ExternMod(decl) => {
                self.res.stack.enter(None);

                let items = decl
                    .items
                    .iter()
                    .flat_map(|item| {
                        let sym = match &item.kind {
                            ItemKind::ExternMod(decl) => &decl.ident,
                            ItemKind::InlineMod(decl) => &decl.ident,
                            ItemKind::Fn(decl) => &decl.ident,
                            ItemKind::ExternFn(decl) => &decl.name,
                        }
                        .sym;

                        self.visit_item(item).map(|id| (sym, id))
                    })
                    .collect();

                self.res.stack.leave();

                session::ItemKind::ExternMod {
                    items,
                    ffi_name: decl.ffi_name.as_ref().map(|i| i.sym),
                    signature_span,
                }
            }
            ItemKind::InlineMod(decl) => {
                self.res.stack.enter(None);

                let items = decl
                    .items
                    .iter()
                    .flat_map(|item| {
                        let sym = match &item.kind {
                            ItemKind::ExternMod(decl) => &decl.ident,
                            ItemKind::InlineMod(decl) => &decl.ident,
                            ItemKind::Fn(decl) => &decl.ident,
                            ItemKind::ExternFn(decl) => &decl.name,
                        }
                        .sym;

                        self.visit_item(item).map(|id| (sym, id))
                    })
                    .collect();

                self.res.stack.leave();

                session::ItemKind::InlineMod {
                    items,
                    signature_span,
                }
            }
        };

        let def_id = self.res.sess.register_item(session::ItemInfo {
            name: sym,
            kind,
            id: item.id,
            vis: match item.vis {
                Visibility::Private => session::Visibility::Private,
                Visibility::Public => session::Visibility::Public,
            },
        });

        self.res.register_def(sym, def_id, item.id);

        Some(def_id)
    }
}

struct SecondPass<'a, 'dcx, 'sess, 'ast> {
    res: &'a mut Resolver<'dcx, 'sess>,

    current_item: Option<&'ast Item>,
}

impl SecondPass<'_, '_, '_, '_> {
    pub fn resolve_segment_in_def(&mut self, seg: &PathSegment, def_id: DefId) -> Def {
        let item = self.res.sess.item(def_id);

        let items = match &item.kind {
            session::ItemKind::InlineMod { items, .. }
            | session::ItemKind::ExternMod { items, .. } => items,
            _ => return Def::Error,
        };

        let result = items
            .get(&seg.ident.sym)
            .map(|id| Def::Def(*id))
            .unwrap_or(Def::Error);

        match result {
            Def::Error => {
                self.res.dcx.emit(
                    Diagnostic::error(self.res.file_id, seg.ident.span, Recovery::Fatal)
                        .with_message(format!(
                            "member `{}` does not exist in module `{}`",
                            seg.ident.sym, item.name
                        ))
                        .with_label(Label::primary(
                            seg.ident.span,
                            "unknown member accessed here",
                        )),
                );
            }
            def @ Def::Def(id) => {
                let member = self.res.sess.item(id);

                if member.vis == session::Visibility::Private && !self.res.stack.is_in_scope(&def) {
                    let message = format!(
                        "member `{}` of `{}` is private and therefore inaccessible in current scope",
                        member.name, item.name
                    );

                    let help = format!("consider making `{}` public", member.name);

                    self.res.dcx.emit(
                        Diagnostic::error(self.res.file_id, seg.ident.span, Recovery::Fatal)
                            .with_message(message)
                            .with_label(Label::help(
                                member.signature_span(),
                                "member is defined here",
                            ))
                            .with_label(Label::primary(seg.ident.span, "attempted access here"))
                            .with_help(help),
                    );

                    return Def::Error;
                }
            }
            Def::Local(_) => {}
        }

        result
    }
}

impl<'ast> Visitor<'ast> for SecondPass<'_, '_, '_, 'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        let old_item = self.current_item.replace(item);

        item.walk(self);

        self.current_item = old_item;
    }

    // We override this visit function so extern modules don't get walked. They
    //   are already registered via `visit_mod_item` or the first pass.
    fn visit_extern_mod_item(&mut self, _extern_module: &'ast ExternModItem) {}

    // We override this visit function so extern functions don't get walked. They
    //   are already registered via `visit_mod_item` or the first pass and would
    //   otherwise register redundant locals for parameters.
    fn visit_extern_fn_item(&mut self, _extern_fun: &'ast ExternFnItem) {}

    fn visit_fn_item(&mut self, fun: &'ast FnItem) {
        let item = self
            .current_item
            .expect("visit_fn_item called outside of item context");

        self.res
            .stack
            .enter(Some(self.res.rcx.defs_by_node[&item.id]));

        fun.walk(self);

        self.res.stack.leave();
    }

    fn visit_mod_item(&mut self, module: &'ast ModItem) {
        let item = self
            .current_item
            .expect("visit_fn_item called outside of item context");

        self.res
            .stack
            .enter(Some(self.res.rcx.defs_by_node[&item.id]));

        for item in &module.items {
            let sym = ident_of_item(item).sym;

            self.res
                .stack
                .register_def(sym, self.res.rcx.defs_by_node[&item.id]);
        }

        module.walk(self);

        self.res.stack.leave();
    }

    fn visit_parameter(&mut self, parameter: &'ast Parameter) {
        let local_id = self.res.sess.register_local(LocalInfo {
            name: parameter.ident.sym,
            defined_where: parameter.span,
            id: parameter.id,
            mutable: false, // for now
        });

        self.res
            .register_local(parameter.ident.sym, local_id, parameter.id);

        parameter.walk(self)
    }

    // We override the visit function so paramter identifiers don't get resolved in the
    //   current scope later in `visit_identifier`.
    fn visit_parameter_identifier(&mut self, _ident: &'ast Ident) {}

    // We override the visit function so paramter identifiers don't get resolved in the
    //   current scope later in `visit_identifier`.
    fn visit_item_identifier(&mut self, _ident: &'ast Ident) {}

    fn visit_block_expr(&mut self, block: &'ast Block) {
        self.res.stack.enter(None);

        block.walk(self);

        self.res.stack.leave();
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ast ayuc_ast::LetStmt) {
        // This is so `let x = x` doesn't reference it's own binding.
        self.visit_expr(&let_stmt.init);

        self.visit_pat(&let_stmt.pat);

        if let Some(ty) = &let_stmt.ty {
            self.visit_ty(ty);
        }
    }

    fn visit_identifier(&mut self, ident: &'ast Ident) {
        if let Some(def) = self.res.stack.lookup(ident.sym) {
            self.res.rcx.name_resolutions.insert(ident.id, def);
        } else {
            self.res.dcx.emit(
                Diagnostic::error(self.res.file_id, ident.span, Recovery::Fatal)
                    .with_message(format!(
                        "unresolved symbol in current scope: `{}`",
                        ident.sym.as_str()
                    ))
                    .with_label(Label::primary(ident.span, "not found in current scope")),
            );

            self.res.rcx.name_resolutions.insert(ident.id, Def::Error);
        }
    }

    fn visit_pat(&mut self, pat: &'ast Pat) {
        match &pat.kind {
            PatKind::Binding(binding) => {
                let local_id = self.res.sess.register_local(LocalInfo {
                    name: binding.sym,
                    defined_where: pat.span,
                    id: pat.id,
                    mutable: binding.mutable,
                });

                self.res.register_local(binding.sym, local_id, pat.id);
            }
            PatKind::Tuple(elements) => elements.walk(self),
        }
    }

    fn visit_literal(&mut self, literal: &'ast Literal) {
        if let Literal::InterpolatedStr { segments, .. } = literal {
            for segment in segments {
                if let IntlSegment::Var(ident) = segment {
                    self.visit_identifier(ident);
                }
            }
        }
    }

    // We override this so path types don't bleed into `visit_path`. Normally,
    //   we would let them bleed into it, because they have to be resolved too,
    //   but Ayu does not yet have custom types.
    fn visit_path_ty(&mut self, _path: &'ast Path) {}

    fn visit_path(&mut self, path: &'ast Path) {
        let first = match path.segments.first() {
            Some(seg) => seg,
            _ => unreachable!(),
        };

        let ident = &first.ident;
        let (def, mut qualified_path) = match self.res.stack.lookup_path(ident.sym) {
            Some(result @ (Def::Def(_) | Def::Local(_), _)) => result,
            Some((Def::Error, _)) => return,
            None => {
                self.res.dcx.emit(
                    Diagnostic::error(self.res.file_id, ident.span, Recovery::Fatal)
                        .with_message(format!(
                            "unresolved symbol in current scope: `{}`",
                            ident.sym.as_str()
                        ))
                        .with_label(Label::primary(ident.span, "not found in current scope")),
                );

                return;
            }
        };

        let def = match def {
            mut def @ Def::Def(_) => {
                let mut remaining = &path.segments[1..];

                self.res.rcx.name_resolutions.insert(first.id, def);

                while let Some(current) = remaining.first() {
                    def = match def {
                        d @ Def::Local(_) => d,
                        Def::Def(id) => self.resolve_segment_in_def(current, id),
                        Def::Error => {
                            break;
                        }
                    };

                    remaining = &remaining[1..];

                    self.res.rcx.name_resolutions.insert(current.id, def);
                    qualified_path.push(def);
                }

                def
            }
            def @ Def::Local(_) => {
                self.res.rcx.name_resolutions.insert(first.id, def);

                def
            }
            Def::Error => unreachable!(),
        };

        // Only necessary for actual items.
        // We make everything absolute, because we first declare all items and then define them after.
        // This allows us to have forward-references even in Lua!
        if let Def::Def(_) = def {
            self.res.rcx.qualified_paths.insert(path.id, qualified_path);
        }

        self.res.rcx.name_resolutions.insert(path.id, def);
    }
}
