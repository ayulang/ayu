use std::collections::HashMap;

use ayuc_ast as ast;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_id::{
    TyId,
    ast::NodeId,
    hir::{DefId, LocalId},
};
use ayuc_session::Session;
use slotmap::SlotMap;

use crate::{Ty, def::Def, scope::ScopeStack};

#[derive(Default)]
pub struct ResolutionContext {
    error_ty: Ty,

    /// Stores the resolved `Ty`s
    pub ty_resolutions: SlotMap<TyId, Ty>,
    pub tys_by_node: HashMap<NodeId, TyId>,

    /// Stores the resolved `Def`s (local or item definitions) of identifiers.
    pub name_resolutions: HashMap<NodeId, Def>,

    /// The assigned [DefId] for identifiers of items.
    pub def_ids: SlotMap<DefId, NodeId>,
    pub defs_by_node: HashMap<NodeId, DefId>,

    /// The assigned [LocalId] for `let` statements.
    pub locals_by_node: HashMap<NodeId, LocalId>,

    pub qualified_paths: HashMap<NodeId, Vec<Def>>,
}

impl ResolutionContext {
    pub fn ty(&self, id: TyId) -> &Ty {
        self.ty_resolutions.get(id).unwrap_or(&self.error_ty)
    }

    pub fn ty_of(&self, id: NodeId) -> &Ty {
        self.tys_by_node
            .get(&id)
            .copied()
            .map(|ty_id| &self.ty_resolutions[ty_id])
            .unwrap_or(&self.error_ty)
    }

    pub fn get_name_res(&self, id: NodeId) -> Def {
        self.name_resolutions
            .get(&id)
            .copied()
            .unwrap_or(Def::Error)
    }
}

pub struct Resolver<'dcx, 'sess> {
    pub(crate) sess: &'sess mut Session,

    pub rcx: ResolutionContext,

    /// For the name resolver.
    pub(crate) stack: ScopeStack,

    /// For diagnostics.
    pub(crate) dcx: &'dcx mut DiagnosticContext,
    pub(crate) file_id: usize,
}

impl<'dcx, 'sess> Resolver<'dcx, 'sess> {
    pub fn new(sess: &'sess mut Session, dcx: &'dcx mut DiagnosticContext, file_id: usize) -> Self {
        Self {
            sess,
            rcx: ResolutionContext::default(),
            stack: ScopeStack::default(),
            dcx,
            file_id,
        }
    }

    /// Constructs a new [Resolver], performs name and type resolution and returns the [ResolutionContext].
    #[inline]
    pub fn resolve(
        sess: &'sess mut Session,
        dcx: &'dcx mut DiagnosticContext,
        file_id: usize,
        ast: &ast::Ast,
    ) -> ResolutionContext {
        let mut this = Self::new(sess, dcx, file_id);

        this.run_name_resolution(ast);

        if this.dcx.requires_abort() {
            return this.rcx;
        }

        this.run_type_resolution(ast);

        this.rcx
    }
}
