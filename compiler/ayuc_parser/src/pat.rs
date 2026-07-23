use ayuc_ast::{Pat, PatBinding, PatKind};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, Keyword, StructuredToken, Token, TokenKind},
};
use ayuc_span::Span;

use crate::{PResult, Parser};

impl Parser<'_, '_, '_> {
    fn parse_pat_ident(&mut self) -> PResult<Pat> {
        let snapshot = self.stream.snapshot();
        let mutable = self.maybe(TokenKind::Keyword(Keyword::Mut));
        let sym = match self.require_token()? {
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(sym),
                ..
            }) => *sym,
            token => {
                let span = token.span();

                return Err(Diagnostic::error(self.file_id, span, Recovery::Fatal)
                    .with_message(format!("expected identifier, got {}", &self.source[span]))
                    .with_label(Label::primary(span, "expected identifier")));
            }
        };

        Ok(Pat {
            span: self.stream.span_since(snapshot),
            id: self.node_id_allocator.allocate(),
            kind: PatKind::Binding(PatBinding { sym, mutable }),
        })
    }

    fn parse_pat_tuple(&mut self) -> PResult<Pat> {
        let StructuredToken::Delimited(span, _, tokens) = self.require_token()? else {
            unreachable!() // Ensured by `parse_pat`
        };

        let mut inner = self.branch(TokenStream::new(tokens));
        let snapshot = inner.stream.snapshot();

        let mut parts = Vec::new();
        let mut expect_pat = !inner.stream.is_exhausted();

        while expect_pat && !inner.stream.is_exhausted() {
            parts.push(inner.parse_pat()?);

            expect_pat = inner.maybe(TokenKind::Comma);
        }

        if expect_pat {
            let span = Span::from(inner.stream.span_since(snapshot).end);

            self.dcx.emit(
                Diagnostic::error(self.file_id, span, Recovery::Recovered)
                    .with_message("expected pattern, got end of input")
                    .with_label(Label::primary(span, "expected pattern")),
            );
        }

        if parts.len() == 1 {
            let part = parts.remove(0);

            self.dcx.emit(
                Diagnostic::warning(self.file_id, *span)
                    .with_message("redundant parenthesis")
                    .with_label(Label::primary(*span, "parenthesis are not needed")),
            );

            Ok(part)
        } else {
            Ok(Pat {
                span: *span,
                id: self.node_id_allocator.allocate(),
                kind: PatKind::Tuple(parts),
            })
        }
    }

    pub fn parse_pat(&mut self) -> PResult<Pat> {
        match self.stream.first() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, _)) => {
                self.parse_pat_tuple()
            }
            Some(StructuredToken::Token(_)) => self.parse_pat_ident(),
            Some(token) => {
                let span = token.span();

                Err(Diagnostic::error(self.file_id, span, Recovery::Fatal)
                    .with_message(format!("expected pattern, got {}", &self.source[span]))
                    .with_label(Label::primary(span, "expected pattern")))
            }
            None => {
                let span = Span::from(self.source.len());

                Err(Diagnostic::error(self.file_id, span, Recovery::Fatal)
                    .with_message("expected pattern, got unexpected end of file")
                    .with_label(Label::primary(span, "expected pattern")))
            }
        }
    }
}
