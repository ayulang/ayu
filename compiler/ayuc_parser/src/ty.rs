use ayuc_ast::{Ty, TyKind};

use crate::{PResult, Parser};

impl Parser<'_, '_, '_> {
    pub fn parse_ty(&mut self) -> PResult<Ty> {
        if let Ok(path) = self.parse_path() {
            Ok(Ty {
                id: self.node_id_allocator.allocate(),
                span: path.span,
                kind: TyKind::Path(path),
            })
        } else {
            todo!()
        }
    }
}
