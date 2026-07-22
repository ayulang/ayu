use ayuc_ast::{Ty, TyKind};
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken, TokenKind},
};

use crate::{PResult, Parser};

impl Parser<'_, '_, '_> {
    fn parse_tuple_ty(&mut self) -> PResult<Ty> {
        let StructuredToken::Delimited(span, Delimiter::Parenthesis, tokens) =
            self.require_token()?
        else {
            unreachable!()
        };

        let mut inner = self.branch(TokenStream::new(tokens));
        let mut parts = Vec::new();
        let mut expect_another = true;

        while expect_another && !inner.stream.is_exhausted() {
            parts.push(inner.parse_ty()?);

            expect_another = inner.maybe(TokenKind::Comma);
        }

        if parts.len() == 1 {
            let part = parts.remove(0);

            self.dcx.emit(
                Diagnostic::warning(self.file_id, *span)
                    .with_message("redundant parenthesis")
                    .with_label(Label::primary(*span, "can be written without parenthesis")),
            );

            Ok(part)
        } else {
            Ok(Ty {
                id: self.node_id_allocator.allocate(),
                span: *span,
                kind: TyKind::Tuple(parts),
            })
        }
    }

    fn parse_path_ty(&mut self) -> PResult<Ty> {
        let path = self.parse_path()?;

        Ok(Ty {
            id: self.node_id_allocator.allocate(),
            span: path.span,
            kind: TyKind::Path(path),
        })
    }

    pub fn parse_ty(&mut self) -> PResult<Ty> {
        match self.stream.first() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, _)) => self.parse_tuple_ty(),
            Some(StructuredToken::Token(_)) => self.parse_path_ty(),
            _ => todo!(),
        }
    }
}
