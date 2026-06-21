use ariadne::{Color, Fmt, Label};
use ayuc_ast::{
    ExternFnDecl,
    expr::{Block, Ident},
    item::{FnDecl, ParameterList},
};
use ayuc_common::{ARIADNE_CONFIG, SourceReport};
use ayuc_lexer::token::{Delimiter, Keyword, StructuredToken};
use ayuc_span::Span;

use crate::{
    Parser,
    parsable::{Parsable, ParseError, Parsed},
};

impl Parsable for FnDecl {
    const NAME: &str = "function declaration";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        parser.assert_keyword(Keyword::Fn)?;

        let ident = parser.assert_parsable::<Ident>()?;

        let params = match ParameterList::parse_with_rollback(parser)? {
            p @ Parsed::Missing(span) => {
                let span = parser.sourced_span(span);

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
            p => p,
        };

        let block = parser.assert_parsable::<Block>()?;

        Ok(Parsed::Present(Self {
            ident,
            block,
            parameters: params.unwrap_or(ParameterList),
        }))
    }
}

impl Parsable for ExternFnDecl {
    const NAME: &str = "extern function declaration";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        parser.assert_keyword(Keyword::Extern)?;
        parser.assert_keyword(Keyword::Fn)?;

        let ident = parser.assert_parsable::<Ident>()?;

        let params = match ParameterList::parse_with_rollback(parser)? {
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
        };

        Ok(Parsed::Present(Self {
            ident,
            parameters: params.unwrap_or(ParameterList),
        }))
    }
}

impl Parsable for ParameterList {
    const NAME: &str = "parameter list";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
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
