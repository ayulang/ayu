use ayuc_ast::{
    ExternFnDecl, Item, ItemKind,
    item::{FnDecl, ParameterList},
};
use ayuc_lexer::token::{Keyword, StructuredToken, Token, TokenKind};

use crate::{PResult, Parser};

impl Parser<'_> {
    pub fn parse_extern_fn_item(&mut self) -> PResult<ExternFnDecl> {
        if !self.maybe(TokenKind::Keyword(Keyword::Extern)) {
            return Err(());
        }

        if !self.maybe(TokenKind::Keyword(Keyword::Fn)) {
            return Err(());
        }

        let ident = self.parse_ident()?;
        let params = self
            .parse_parameter_list()
            .unwrap_or(ParameterList::default());

        /*let params = match ParameterList::parse_with_rollback(parser)? {
            p @ Parsed::Missing(span) => {
                let span = parser.sourced_span(span);

                parser.session.emit(
                    SourceReport::build(ariadne::ReportKind::Error, span)
                        .with_config(ARIADNE_CONFIG)
                        .with_message("extern function declarations require a parameter list")
                        .with_label(
                            Label::new(span)
                                .with_color(ariadne::Color::BrightRed)
                                .with_message("missing parameter list".fg(Color::BrightRed)),
                        )
                        .with_help(format!(
                            "add a parameter list: `{}{}`",
                            ident.sym.as_str(),
                            "()".fg(Color::BrightGreen)
                        ))
                        .finish(),
                );

                p
            }
            p => p,
        };*/

        if self.maybe(TokenKind::Arrow) {
            let _path = self.parse_path()?;
        }

        Ok(ExternFnDecl {
            ident,
            parameters: params,
        })
    }

    pub fn parse_fn_item(&mut self) -> PResult<FnDecl> {
        if !self.maybe(TokenKind::Keyword(Keyword::Fn)) {
            return Err(());
        }

        let ident = self.parse_ident()?;

        /*let params = match ParameterList::parse_with_rollback(self).unwrap() {
            p @ Parsed::Missing(span) => {
                let span = self.sourced_span(span);

                parser.session.emit(
                    SourceReport::build(ariadne::ReportKind::Error, span)
                        .with_config(ARIADNE_CONFIG)
                        .with_message("function declarations require a parameter list")
                        .with_label(
                            Label::new(span)
                                .with_color(ariadne::Color::BrightRed)
                                .with_message("missing parameter list".fg(Color::BrightRed)),
                        )
                        .with_help(format!(
                            "add a parameter list: `{}{}`",
                            ident.sym.as_str(),
                            "()".fg(Color::BrightGreen)
                        ))
                        .finish(),
                );

                p
            }
            p => ,
        };*/

        let params = self
            .parse_parameter_list()
            .unwrap_or(ParameterList::default());

        if self.maybe(TokenKind::Arrow) {
            let _path = self.parse_path()?;
        }

        let block = self.parse_block_expr()?;

        Ok(FnDecl {
            ident,
            block,
            parameters: params,
        })
    }

    pub fn parse_item(&mut self) -> PResult<Item> {
        let Some(first) = self.stream.first() else {
            return Err(());
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
