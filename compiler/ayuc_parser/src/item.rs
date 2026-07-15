use ayuc_ast::{ExternFnDecl, Item, ItemKind, ParameterList, Ty, TyKind, Visibility, item::FnDecl};
use ayuc_diagnostic::{Diagnostic, Label, colored::Colorize};
use ayuc_lexer::token::{Keyword, StructuredToken, Token, TokenKind};
use ayuc_span::Span;

use crate::{PResult, Parser};

impl Parser<'_, '_> {
    pub fn parse_extern_fn_item(&mut self) -> PResult<ExternFnDecl> {
        if !self.maybe(TokenKind::Keyword(Keyword::Extern)) {
            unreachable!()
        }

        if !self.maybe(TokenKind::Keyword(Keyword::Fn)) {
            unreachable!()
        }

        let (ffi_name, name) = {
            let first_ident = self.parse_ident()?; // always there

            if self.maybe(TokenKind::Keyword(Keyword::As)) {
                (Some(first_ident), self.parse_ident()?)
            } else {
                (None, first_ident)
            }
        };

        let params = match self.with_rollback(|this| this.parse_parameter_list()) {
            Ok(list) => list,
            Err(_) => {
                let span = Span::from(name.span.end);

                self.dcx.emit(
                    Diagnostic::error(self.file_id, span)
                        .with_message("missing parameter list")
                        .with_label(Label::primary(
                            span,
                            "extern function items require a parameter list",
                        ))
                        .with_help(format!(
                            "consider adding a parameter list: {}{}",
                            name.sym.as_str().dimmed(),
                            "()".bright_green()
                        )),
                );

                ParameterList::default()
            }
        };

        let ty = if self.maybe(TokenKind::Arrow) {
            self.parse_ty()?
        } else {
            Ty {
                id: self.node_id_allocator.allocate(),
                span: params.span.end.into(),
                kind: TyKind::Unit,
            }
        };

        Ok(ExternFnDecl {
            name,
            ffi_name,
            parameters: params,
            return_ty: ty,
        })
    }

    pub fn parse_fn_item(&mut self) -> PResult<FnDecl> {
        if !self.maybe(TokenKind::Keyword(Keyword::Fn)) {
            unreachable!()
        }

        let ident = self.parse_ident()?;
        let params = match self.with_rollback(|this| this.parse_parameter_list()) {
            Ok(list) => list,
            Err(_) => {
                let span = Span::from(ident.span.end);

                self.dcx.emit(
                    Diagnostic::error(self.file_id, span)
                        .with_message("missing parameter list")
                        .with_label(Label::primary(
                            span,
                            "function items require a parameter list",
                        ))
                        .with_help(format!(
                            "consider adding a parameter list: {}{}",
                            ident.sym.as_str().dimmed(),
                            "()".bright_green()
                        )),
                );

                ParameterList::default()
            }
        };

        let ty = if self.maybe(TokenKind::Arrow) {
            self.parse_ty()?
        } else {
            Ty {
                id: self.node_id_allocator.allocate(),
                span: params.span.end.into(),
                kind: TyKind::Unit,
            }
        };

        let block = self.parse_block_expr()?;

        Ok(FnDecl {
            ident,
            block,
            parameters: params,
            return_ty: ty,
        })
    }

    pub fn parse_item(&mut self) -> PResult<Item> {
        let snapshot = self.stream.snapshot();
        let vis = if self.maybe(TokenKind::Keyword(Keyword::Pub)) {
            Visibility::Public
        } else {
            Visibility::Private
        };

        let Some(first) = self.stream.first() else {
            todo!()
        };

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
            vis,
            id,
            span: self.stream.span_since(snapshot),
            kind,
        })
    }
}
