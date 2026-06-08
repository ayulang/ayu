use ayuc_hir::{id::ModuleId, module::Module};

pub struct TyCtx {
    pub modules: Vec<Module>,
    pub next_module_id: usize,
}

impl TyCtx {
    pub fn register_module(&mut self, module: Module) {
        self.modules.push(module);
    }

    pub fn module(&self, id: ModuleId) -> &Module {
        &self.modules[id.get()]
    }

    pub fn mint_module_id(&mut self) -> ModuleId {
        self.next_module_id += 1;

        ModuleId::new(self.next_module_id - 1)
    }
}
