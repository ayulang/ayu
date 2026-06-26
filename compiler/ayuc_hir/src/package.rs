use ayuc_id::hir::{DefId, HirIdAllocator, LocalId, PackageId};
use slotmap::SlotMap;

use crate::{Local, item::Item};

#[derive(Debug)]
pub struct Package {
    pub id: PackageId,

    pub items: SlotMap<DefId, Item>,
    pub locals: SlotMap<LocalId, Local>,

    pub hir_id_allocator: HirIdAllocator,
}

impl Package {
    pub fn new(id: PackageId) -> Self {
        Self {
            id,
            items: SlotMap::with_key(),
            locals: SlotMap::with_key(),
            hir_id_allocator: HirIdAllocator::new(),
        }
    }
}
