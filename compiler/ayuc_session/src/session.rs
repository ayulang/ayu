use ayuc_id::hir::{DefId, LocalId};
use slotmap::SlotMap;

use crate::{ItemInfo, local::LocalInfo};

#[derive(Default)]
pub struct Session {
    items: SlotMap<DefId, ItemInfo>,
    locals: SlotMap<LocalId, LocalInfo>,
}

impl Session {
    pub fn register_item(&mut self, info: ItemInfo) -> DefId {
        self.items.insert(info)
    }

    pub fn register_local(&mut self, info: LocalInfo) -> LocalId {
        self.locals.insert(info)
    }

    pub fn item(&self, id: DefId) -> &ItemInfo {
        &self.items[id]
    }

    pub fn local(&self, id: LocalId) -> &LocalInfo {
        &self.locals[id]
    }
}
