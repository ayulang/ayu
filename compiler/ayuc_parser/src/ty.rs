use ayuc_ast::{Ty, TyKind};

use crate::{PResult, Parser};

impl Parser<'_> {
    pub fn parse_ty(&mut self) -> PResult<Ty> {
        let snapshot = self.stream.snapshot();

        if let Ok(path) = self.parse_path() {
            Ok(Ty {
                id: self.node_id_allocator.allocate(),
                span: self.stream.span_since(snapshot),
                kind: TyKind::Path(path),
            })
        } else {
            Err(())
        }
    }
}
