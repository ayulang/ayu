/// Contains implementations of the [Parsable] trait for `ayuc_ir` nodes.
pub mod impls;
pub mod parsable;

use ariadne::{Color, Fmt, Label, Report};
use ayuc_common::SourceReport;
use ayuc_ir::{Ast, node::stmt::fn_decl::FnDecl};
use ayuc_lexer::{
    stream::{self, TokenStream},
    token::{Keyword, StructuredToken, TokenKind},
};
use ayuc_source::SourceSpan;
use ayuc_span::Span;

use crate::parsable::{Parsable, ParseError, ParseResult};

/// Used for parsing an input file into an abstract syntax tree.
pub struct Parser<'a> {
    /// The input token stream.
    pub(crate) stream: TokenStream,

    file_id: usize,
    source: &'a str,

    /// The produced diagnostics from both the parser and lexer.
    pub diagnostics: Vec<Report<'a, SourceSpan>>,
}

impl<'a> Parser<'a> {
    /// This function only errors if an unrecoverable error occurred while lexing the input.
    pub fn new(file_id: usize, source: &'a str) -> Result<Self, SourceReport<'a>> {
        let (stream, diagnostics) = ayuc_lexer::lex(file_id, source)?;

        Ok(Self {
            stream,
            file_id,
            source,
            diagnostics,
        })
    }

    pub(crate) fn sourced_span(&self, span: Span) -> SourceSpan {
        SourceSpan::new(self.file_id, span)
    }

    pub(crate) fn expect_token(&mut self) -> ParseResult<'a, StructuredToken> {
        if let Some(token) = self.stream.consume() {
            Ok(token)
        } else {
            let sourced = self.sourced_span(Span::from(self.stream.last_position.end));

            Err(ParseError::new().with_report(
                Report::build(ariadne::ReportKind::Error, sourced)
                    .with_label(
                        Label::new(sourced)
                            .with_color(Color::BrightRed)
                            .with_message("unexpected end of stream".fg(Color::BrightRed)),
                    )
                    .finish(),
            ))
        }
    }

    pub(crate) fn expect<P: Parsable>(&mut self) -> ParseResult<'a, P> {
        P::parse(self)
    }

    pub fn parse_full(&mut self) -> Ast {
        while let Some(token) = self.stream.first() {
            match token {
                StructuredToken::Token(tok) if tok.kind == TokenKind::Keyword(Keyword::Fn) => {
                    self.stream.consume();

                    if let Err(err) = self.expect::<FnDecl>() {
                        if let Some(report) = err.report {
                            self.diagnostics.push(report);
                        }
                    }
                }
                _ => {
                    self.stream.consume();
                }
            }
        }

        todo!()
    }
}
