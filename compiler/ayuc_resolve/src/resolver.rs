use std::collections::HashMap;

use ayuc_ast as ast;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_id::{
    ast::NodeId,
    hir::{DefId, LocalId},
};
use slotmap::SlotMap;

use crate::{def::Def, scope::ScopeStack, ty::Ty};

#[derive(Default)]
pub struct ResolutionContext {
    /// Stores the resolved `Ty`s of AST `Ty`s
    pub ty_resolutions: HashMap<NodeId, Ty>,

    /// Stores the resolved `Def`s (local or item definitions) of identifiers.
    pub name_resolutions: HashMap<NodeId, Def>,

    /// The assigned [DefId] for identifiers of items.
    pub def_ids: SlotMap<DefId, NodeId>,
    pub defs_by_node: HashMap<NodeId, DefId>,

    /// The assigned [LocalId] for `let` statements.
    pub locals: SlotMap<LocalId, NodeId>,
    pub locals_by_node: HashMap<NodeId, LocalId>,
}

impl ResolutionContext {
    pub fn get_ty_res(&self, id: NodeId) -> Ty {
        self.ty_resolutions.get(&id).copied().unwrap_or(Ty::Error)
    }

    pub fn get_name_res(&self, id: NodeId) -> Def {
        self.name_resolutions
            .get(&id)
            .copied()
            .unwrap_or(Def::Error)
    }
}

pub struct Resolver<'dcx> {
    pub rcx: ResolutionContext,

    /// For the name resolver.
    pub(crate) stack: ScopeStack,

    /// For diagnostics.
    pub(crate) dcx: &'dcx mut DiagnosticContext,
    pub(crate) file_id: usize,
}

impl<'dcx> Resolver<'dcx> {
    pub fn new(dcx: &'dcx mut DiagnosticContext, file_id: usize) -> Self {
        Self {
            rcx: ResolutionContext::default(),
            stack: ScopeStack::new(),
            dcx,
            file_id,
        }
    }

    /// Constructs a new [Resolver], performs name and type resolution and returns the [ResolutionContext].
    #[inline]
    pub fn resolve(
        dcx: &'dcx mut DiagnosticContext,
        file_id: usize,
        ast: &ast::Ast,
    ) -> ResolutionContext {
        let mut this = Self::new(dcx, file_id);

        this.resolve_names(ast);
        this.resolve_types(ast);

        this.rcx
    }
}
