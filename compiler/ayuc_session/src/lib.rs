pub mod item;

use ayuc_id::hir::DefId;
use slotmap::SlotMap;

use crate::item::ItemInfo;

#[derive(Default)]
pub struct Session {
    items: SlotMap<DefId, ItemInfo>,
}

impl Session {
    pub fn register_item(&mut self, info: ItemInfo) -> DefId {
        self.items.insert(info)
    }

    pub fn item(&self, id: DefId) -> &ItemInfo {
        &self.items[id]
    }
}
