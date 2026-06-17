/// Contains implementations of the [Parsable] trait for `ayuc_ir` nodes.
pub mod impls;
pub mod parsable;
pub mod session;

use ariadne::{Color, Fmt, Label, ReportKind};
use ayuc_ast::{Ast, Expression, Literal, item::Item, stmt::Statement};
use ayuc_common::{ARIADNE_CONFIG, SourceReport};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, Keyword, StructuredToken, Token, TokenKind},
};
use ayuc_source::SourceSpan;
use ayuc_span::{Span, symbol::Symbol};

use crate::{
    parsable::{Assertable, ParseError, Parsed},
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

    pub(crate) fn assert_parsable<P: Assertable>(&mut self) -> Result<P, ParseError> {
        match P::parse(self)? {
            Parsed::Present(p) => Ok(p),
            Parsed::Missing(span) => {
                let span = self.sourced_span(span);

                self.session.emit(
                    SourceReport::build(ariadne::ReportKind::Error, span)
                        .with_config(ARIADNE_CONFIG)
                        .with_message(format!(
                            "expected {}, got: `{}`",
                            P::NAME,
                            &self.source[span]
                        ))
                        .with_label(
                            Label::new(span)
                                .with_color(ariadne::Color::BrightRed)
                                .with_message(format!("expected {}", P::NAME).fg(Color::BrightRed)),
                        )
                        .finish(),
                );

                Err(ParseError::Unrecoverable)
            }
        }
    }

    pub(crate) fn assert_keyword(&mut self, keyword: Keyword) -> Result<(), ParseError> {
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

            Err(ParseError::Unrecoverable)
        }
    }

    pub(crate) fn sourced_span(&self, span: Span) -> SourceSpan {
        SourceSpan::new(self.file_id, span)
    }

    pub(crate) fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let Some(first) = self.stream.first() else {
            return Err(ParseError::Unrecoverable);
        };

        match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Literal { data_span },
                span,
            }) => {
                let expr = Expression::Lit(Literal::Str {
                    span: *span,
                    data: Symbol::intern(&self.source[data_span]),
                });

                self.stream.consume();

                Ok(expr)
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) if matches!(
                self.stream.second(),
                Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, _))
            ) =>
            {
                Ok(Expression::Call(self.assert_parsable()?))
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) => Ok(Expression::Identifier(self.assert_parsable()?)),
            _ => todo!(),
        }
    }

    pub(crate) fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let Some(first) = self.stream.first() else {
            return Err(ParseError::Unrecoverable);
        };

        // redo this so it tries known patterns first, and then maybe expressions?
        match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) if matches!(
                self.stream.second(),
                Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, _))
            ) =>
            {
                Ok(Statement::Expr(ayuc_ast::Expression::Call(
                    self.assert_parsable()?,
                )))
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Let),
                ..
            }) => Ok(Statement::VarDecl(self.assert_parsable()?)),
            _ => todo!(),
        }
    }

    pub(crate) fn parse_item(&mut self) -> Result<Item, ParseError> {
        let Some(first) = self.stream.first() else {
            return Err(ParseError::Unrecoverable);
        };

        match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Fn),
                ..
            }) => Ok(Item::Fn(self.assert_parsable()?)),
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
                Ok(Item::ExternFn(self.assert_parsable()?))
            }
            _ => todo!(),
        }
    }

    pub fn parse_full(mut self) -> (Option<Ast>, ParseSession<'a>) {
        let mut items = Vec::new();

        while !self.stream.is_exhausted()
            && !matches!(
                self.stream.first(),
                Some(StructuredToken::Token(Token {
                    kind: TokenKind::Eof,
                    ..
                }))
            )
        {
            match self.parse_item() {
                Ok(node) => items.push(node),
                Err(_) => return (None, self.session),
            }
        }

        (Some(Ast { items }), self.session)
    }
}
