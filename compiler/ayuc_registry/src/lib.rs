//! This module provides a registry for storing relevant information tied to a ModuleId.
//! It's used to retrieve module information from a [ModuleId] and/or absolute file path.

use ayuc_ast::Ast;
use ayuc_hir::Module;
use ayuc_id::{ModuleId, ast::NodeId};
use bimap::BiHashMap;
use slotmap::{SecondaryMap, SlotMap};

/// A registry containing relevant information for modules such as their [Ast], [Module] and module dependencies.
/// It also has a field to look up [`ModuleId`]s by their absolute file path.
#[derive(Default)]
pub struct ModuleRegistry {
    /// A [SlotMap] containing an AST for a module. May be [`None`] if the file couldn't be parsed.
    pub trees: SlotMap<ModuleId, Option<Ast>>,
    /// A [SecondaryMap] storing the HIR module for a [`ModuleId`].
    pub modules: SecondaryMap<ModuleId, Module>,
    /// A [SecondaryMap] that has a per-module list of all [`ModuleId`]s that a they are dependant of paired with the
    ///   [NodeId] of the statement that declared the dependency.
    pub dependencies: SecondaryMap<ModuleId, Vec<(NodeId, ModuleId)>>,

    /// A [HashMap] that ties all existing [`ModuleId`]s to their absolute file path.
    pub id_by_path: BiHashMap<String, ModuleId>,
}

impl ModuleRegistry {
    /// Adds an Ast to the registry and returns the associated [ModuleId].
    pub fn add_ast(&mut self, path: String, ast: Ast) -> ModuleId {
        let id = self.trees.insert(Some(ast));

        self.id_by_path.insert(path, id);

        id
    }

    /// Adds an Ast to the registry that failed to parse and returns the associated [ModuleId].
    pub fn add_failed_ast(&mut self, path: String) -> ModuleId {
        let id = self.trees.insert(None);

        self.id_by_path.insert(path, id);

        id
    }
}
