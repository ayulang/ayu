use crate::{DefId, id::PackageId, item::Item};

#[derive(Debug)]
pub struct Package {
    pub id: PackageId,
    pub items: Vec<Item>,

    pub next_def_id: usize,
}

impl Package {
    pub fn mint_def_id(&mut self) -> DefId {
        self.next_def_id += 1;

        DefId::new(self.next_def_id)
    }
}
