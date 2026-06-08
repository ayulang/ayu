use crate::{id::ModuleId, item::Item};

#[derive(Debug)]
pub struct Module {
    pub id: ModuleId,
    pub items: Vec<Item>,
}
