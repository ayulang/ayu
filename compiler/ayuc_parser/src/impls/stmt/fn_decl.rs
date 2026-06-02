use ariadne::{Color, Fmt, Label};
use ayuc_common::SourceReport;
use ayuc_ir::node::{
    leaf::ident::Ident,
    stmt::fn_decl::{FnDecl, ParameterList},
};
use ayuc_lexer::token::{Delimiter, StructuredToken};
use ayuc_span::Span;

use crate::{
    Parser,
    parsable::{Parsable, Parsed},
    session::ParseSession,
};

impl Parsable for FnDecl {
    fn parse<'a>(parser: &mut Parser<'a>, sess: &mut ParseSession<'a>) -> Result<Parsed<Self>, ()> {
        let ident = match Ident::parse(parser, sess)? {
            Parsed::Present(ident) => ident,
            Parsed::Missing(span) => {
                let span = parser.sourced_span(span);

                sess.emit(
                    SourceReport::build(ariadne::ReportKind::Error, span)
                        .with_message(format!(
                            "expected identifier, got: `{}`",
                            &parser.source[span]
                        ))
                        .with_label(
                            Label::new(span)
                                .with_color(ariadne::Color::BrightRed)
                                .with_message("expected an identifier".fg(Color::BrightRed)),
                        )
                        .finish(),
                );

                return Err(());
            }
        };

        let _params = match ParameterList::parse(parser, sess)? {
            p @ Parsed::Missing(span) => {
                let span = parser.sourced_span(span);

                sess.emit(
                    SourceReport::build(ariadne::ReportKind::Error, span)
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
            p => p,
        };

        Err(())
    }
}

impl Parsable for ParameterList {
    fn parse<'a>(
        parser: &mut Parser<'a>,
        _sess: &mut ParseSession<'a>,
    ) -> Result<Parsed<Self>, ()> {
        let snapshot = parser.stream.snapshot();

        let _tokens = match parser.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => {
                return Ok(Parsed::Missing(Span::from(
                    parser.stream.past_span_or_distance(2, snapshot).end,
                )));
            }
        };

        Ok(Parsed::Present(Self))
    }
}
