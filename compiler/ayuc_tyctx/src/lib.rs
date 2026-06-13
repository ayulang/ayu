use ayuc_hir::{id::PackageId, package::Package};

pub struct TyCtx {
    pub packages: Vec<Package>,
    pub next_package_id: usize,
}

impl TyCtx {
    pub fn register_package(&mut self, module: Package) {
        self.packages.push(module);
    }

    pub fn package(&self, id: PackageId) -> &Package {
        &self.packages[id.get()]
    }

    pub fn mint_package_id(&mut self) -> PackageId {
        self.next_package_id += 1;

        PackageId::new(self.next_package_id - 1)
    }
}
