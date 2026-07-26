use std::collections::HashMap;

use ayuc_ast as ast;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_id::{
    ModuleId, TyId,
    ast::NodeId,
    hir::{DefId, LocalId},
};
use ayuc_registry::ModuleRegistry;
use ayuc_session::Session;
use ayuc_type::{interner::TypeInterner, ty::TyKind};
use slotmap::SlotMap;

use crate::{def::Def, scope::ScopeStack};

pub struct ResolutionContext {
    error_id: TyId,

    /// Stores the resolved `Ty`s
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
    pub fn new(error_id: TyId) -> Self {
        Self {
            error_id,
            tys_by_node: HashMap::default(),
            name_resolutions: HashMap::default(),
            def_ids: SlotMap::default(),
            defs_by_node: HashMap::default(),
            locals_by_node: HashMap::default(),
            qualified_paths: HashMap::default(),
        }
    }

    pub fn ty_id_of(&self, id: NodeId) -> TyId {
        self.tys_by_node.get(&id).copied().unwrap_or(self.error_id)
    }

    pub fn ty_of<'a>(&self, id: NodeId, interner: &'a TypeInterner) -> &'a TyKind {
        interner.get(self.ty_id_of(id))
    }

    pub fn get_name_res(&self, id: NodeId) -> Def {
        self.name_resolutions
            .get(&id)
            .copied()
            .unwrap_or(Def::Error)
    }

    #[inline]
    pub fn is_error(&self, id: TyId) -> bool {
        id == self.error_id
    }
}

pub struct Resolver<'dcx, 'sess, 'reg> {
    pub(crate) reg: &'reg ModuleRegistry,
    pub(crate) sess: &'sess mut Session,

    pub rcx: ResolutionContext,

    /// For the name resolver.
    pub(crate) stack: ScopeStack,

    /// For diagnostics.
    pub(crate) dcx: &'dcx mut DiagnosticContext,
    pub(crate) file_id: usize,

    pub(crate) current_module: ModuleId,
}

impl<'dcx, 'sess, 'reg> Resolver<'dcx, 'sess, 'reg> {
    pub fn new(
        reg: &'reg ModuleRegistry,
        sess: &'sess mut Session,
        dcx: &'dcx mut DiagnosticContext,
        file_id: usize,
        current_module: ModuleId,
    ) -> Self {
        let error_id = sess.interner.intern(TyKind::Error);

        Self {
            reg,
            sess,
            rcx: ResolutionContext::new(error_id),
            stack: ScopeStack::default(),
            dcx,
            file_id,
            current_module,
        }
    }

    /// Constructs a new [Resolver], performs name and type resolution and returns the [ResolutionContext].
    #[inline]
    pub fn resolve(
        reg: &'reg ModuleRegistry,
        sess: &'sess mut Session,
        dcx: &'dcx mut DiagnosticContext,
        file_id: usize,
        ast: &ast::Ast,
        current_module: ModuleId,
    ) -> ResolutionContext {
        let mut this = Self::new(reg, sess, dcx, file_id, current_module);

        this.run_name_resolution(ast);

        if this.dcx.requires_abort() {
            return this.rcx;
        }

        this.run_type_resolution(ast);

        this.rcx
    }

    pub fn ty(&self, id: TyId) -> &TyKind {
        self.sess.interner.get(id)
    }

    pub fn ty_of(&self, id: NodeId) -> &TyKind {
        self.ty(self.rcx.ty_id_of(id))
    }
}
