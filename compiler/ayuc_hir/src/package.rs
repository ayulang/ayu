use crate::{DefId, HirIdAllocator, id::PackageId, item::Item};

#[derive(Debug)]
pub struct Package {
    pub id: PackageId,
    pub items: Vec<Item>,

    pub hir_id_allocator: HirIdAllocator,
    pub next_def_id: usize,
}

impl Package {
    pub fn mint_def_id(&mut self) -> DefId {
        self.next_def_id += 1;

        DefId::new(self.next_def_id)
    }
}
