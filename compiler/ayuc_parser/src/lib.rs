/// Contains implementations of the [Parsable] trait for `ayuc_ir` nodes.
pub mod impls;
pub mod parsable;
pub mod session;

use ariadne::{Color, Fmt, Label, ReportKind};
use ayuc_common::{ARIADNE_CONFIG, SourceReport};
use ayuc_ir::{
    Ast,
    node::{
        Node,
        decl::{Declaration, function::FnDecl},
    },
};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Keyword, StructuredToken, Token, TokenKind},
};
use ayuc_source::SourceSpan;
use ayuc_span::Span;

use crate::{
    parsable::{Assertable, Parsable, Parsed},
    session::ParseSession,
};

/// Used for parsing an input file into an abstract syntax tree.
pub struct Parser<'a> {
    /// The input token stream.
    pub(crate) stream: TokenStream<'a>,

    file_id: usize,
    source: &'a str,

    pub(crate) session: ParseSession<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(file_id: usize, source: &'a str, stream: TokenStream<'a>) -> Self {
        Self {
            stream,
            file_id,
            source,
            session: ParseSession::default(),
        }
    }

    pub fn extend_diagnostics(&mut self, diagnostics: Vec<SourceReport<'a>>) {
        self.session.extend(diagnostics);
    }

    pub fn branch(&self, stream: TokenStream<'a>) -> Self {
        Self::new(self.file_id, self.source, stream)
    }

    pub(crate) fn assert_parsable<P: Assertable>(&mut self) -> Result<P, ()> {
        match P::parse(self)? {
            Parsed::Present(p) => Ok(p),
            Parsed::Missing(span) => {
                let span = self.sourced_span(span);

                self.session.emit(
                    SourceReport::build(ariadne::ReportKind::Error, span)
                        .with_config(ARIADNE_CONFIG)
                        .with_message(format!(
                            "expected identifier, got: `{}`",
                            &self.source[span]
                        ))
                        .with_label(
                            Label::new(span)
                                .with_color(ariadne::Color::BrightRed)
                                .with_message("expected an identifier".fg(Color::BrightRed)),
                        )
                        .finish(),
                );

                Err(())
            }
        }
    }

    pub(crate) fn assert_keyword(&mut self, keyword: Keyword) -> Result<(), ()> {
        let snapshot = self.stream.snapshot();

        if let Some(StructuredToken::Token(Token {
            kind: TokenKind::Keyword(kw),
            ..
        })) = self.stream.consume()
            && *kw == keyword
        {
            Ok(())
        } else {
            let past_span = self.stream.span_since(snapshot);
            let sourced_span = self.sourced_span(past_span);

            self.session.emit(
                SourceReport::build(ReportKind::Error, sourced_span)
                    .with_config(ARIADNE_CONFIG)
                    .with_message(format!(
                        "expected `{keyword}` keyword, got: `{}`",
                        &self.source[past_span]
                    ))
                    .with_label(
                        Label::new(sourced_span)
                            .with_color(Color::BrightRed)
                            .with_message(
                                format!("expected `{keyword}` keyword").fg(Color::BrightRed),
                            ),
                    )
                    .finish(),
            );

            Err(())
        }
    }

    pub(crate) fn sourced_span(&self, span: Span) -> SourceSpan {
        SourceSpan::new(self.file_id, span)
    }

    pub(crate) fn parse_node(&mut self) -> Result<Node, ()> {
        match self.stream.first() {
            Some(StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Fn),
                ..
            })) => {
                let decl = match FnDecl::parse(self)? {
                    Parsed::Missing(_) => return Err(()),
                    Parsed::Present(decl) => decl,
                };

                Ok(Node::Decl(Declaration::Function(decl)))
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn parse_full(mut self) -> (Option<Ast>, ParseSession<'a>) {
        let mut nodes = Vec::new();

        while !self.stream.is_exhausted() {
            match self.parse_node() {
                Ok(node) => nodes.push(node),
                Err(()) => return (None, self.session),
            }
        }

        (Some(Ast { nodes }), self.session)
    }
}
