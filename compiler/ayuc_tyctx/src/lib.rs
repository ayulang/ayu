use ayuc_hir::package::Package;
use ayuc_id::hir::PackageId;

pub struct TyCtx {
    pub packages: Vec<Package>,
    pub next_package_id: usize,
}

impl TyCtx {
    pub fn register_package(&mut self, package: Package) -> PackageId {
        let id = package.id;

        self.packages.push(package);

        id
    }

    pub fn package(&self, id: PackageId) -> &Package {
        &self.packages[id.get()]
    }

    pub fn mint_package_id(&mut self) -> PackageId {
        self.next_package_id += 1;

        PackageId::new(self.next_package_id - 1)
    }
}
