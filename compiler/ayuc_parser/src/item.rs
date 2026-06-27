use ayuc_ast::{ExternFnDecl, Item, ItemKind, Ty, TyKind, item::FnDecl};
use ayuc_lexer::token::{Keyword, StructuredToken, Token, TokenKind};

use crate::{PResult, Parser};

impl Parser<'_, '_> {
    pub fn parse_extern_fn_item(&mut self) -> PResult<ExternFnDecl> {
        if !self.maybe(TokenKind::Keyword(Keyword::Extern)) {
            return Err(crate::DummyError);
        }

        if !self.maybe(TokenKind::Keyword(Keyword::Fn)) {
            return Err(crate::DummyError);
        }

        let ident = self.parse_ident()?;
        let params = self.parse_parameter_list().unwrap_or_default();

        let snapshot = self.stream.snapshot();

        let ty_kind = if self.maybe(TokenKind::Arrow) {
            let path = self.parse_path()?;

            TyKind::Path(path)
        } else {
            TyKind::Unit
        };

        Ok(ExternFnDecl {
            ident,
            parameters: params,
            return_ty: Ty {
                id: self.node_id_allocator.allocate(),
                span: self.stream.span_since(snapshot),
                kind: ty_kind,
            },
        })
    }

    pub fn parse_fn_item(&mut self) -> PResult<FnDecl> {
        if !self.maybe(TokenKind::Keyword(Keyword::Fn)) {
            return Err(crate::DummyError);
        }

        let ident = self.parse_ident()?;
        let params = self.parse_parameter_list().unwrap_or_default();

        let snapshot = self.stream.snapshot();

        let ty_kind = if self.maybe(TokenKind::Arrow) {
            let path = self.parse_path()?;

            TyKind::Path(path)
        } else {
            TyKind::Unit
        };

        let block = self.parse_block_expr()?;

        Ok(FnDecl {
            ident,
            block,
            parameters: params,
            return_ty: Ty {
                id: self.node_id_allocator.allocate(),
                span: self.stream.span_since(snapshot),
                kind: ty_kind,
            },
        })
    }

    pub fn parse_item(&mut self) -> PResult<Item> {
        let Some(first) = self.stream.first() else {
            return Err(crate::DummyError);
        };

        let snapshot = self.stream.snapshot();

        let (id, kind) = match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Fn),
                ..
            }) => (
                self.node_id_allocator.allocate(),
                ItemKind::Fn(self.parse_fn_item()?),
            ),
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Extern),
                ..
            }) if matches!(
                self.stream.second(),
                Some(StructuredToken::Token(Token {
                    kind: TokenKind::Keyword(Keyword::Fn),
                    ..
                }))
            ) =>
            {
                (
                    self.node_id_allocator.allocate(),
                    ItemKind::ExternFn(self.parse_extern_fn_item()?),
                )
            }
            _ => todo!(),
        };

        Ok(Item {
            id,
            span: self.stream.span_since(snapshot),
            kind,
        })
    }
}
