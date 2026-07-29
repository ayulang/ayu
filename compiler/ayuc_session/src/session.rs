use std::collections::{HashMap, HashSet};

use ayuc_id::{
    TyId,
    ast::NodeId,
    hir::{DefId, LocalId},
};
use ayuc_type::interner::TypeInterner;
use slotmap::{SecondaryMap, SlotMap};

use crate::{ItemInfo, local::LocalInfo};

#[derive(Default)]
pub struct Session {
    pub interner: TypeInterner,

    items: SlotMap<DefId, ItemInfo>,
    item_ty: SecondaryMap<DefId, TyId>,

    locals: SlotMap<LocalId, LocalInfo>,
    locals_by_id: HashMap<NodeId, LocalId>,

    synthetics: HashSet<NodeId>,
}

impl Session {
    pub fn mark_as_synthetic(&mut self, id: NodeId) {
        self.synthetics.insert(id);
    }

    pub fn is_synthetic(&self, id: NodeId) -> bool {
        self.synthetics.contains(&id)
    }

    pub fn register_item(&mut self, info: ItemInfo) -> DefId {
        self.items.insert(info)
    }

    pub fn register_item_ty(&mut self, id: DefId, ty: TyId) {
        self.item_ty.insert(id, ty);
    }

    pub fn maybe_item_ty(&self, id: DefId) -> Option<TyId> {
        self.item_ty.get(id).copied()
    }

    pub fn register_local(&mut self, info: LocalInfo) -> LocalId {
        let node_id = info.id;
        let id = self.locals.insert(info);

        self.locals_by_id.insert(node_id, id);

        id
    }

    pub fn item(&self, id: DefId) -> &ItemInfo {
        &self.items[id]
    }

    pub fn local(&self, id: LocalId) -> &LocalInfo {
        &self.locals[id]
    }
}
