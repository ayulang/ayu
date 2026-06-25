use ariadne::{Color, Fmt, Label};
use ayuc_ast::{
    ExternFnDecl, Parameter, Path, Ty,
    expr::{Block, Ident},
    item::{FnDecl, ParameterList},
};
use ayuc_common::{ARIADNE_CONFIG, SourceReport};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, Keyword, StructuredToken, TokenKind},
};
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

        if parser.maybe(TokenKind::Arrow) {
            let _path = match Path::parse_with_rollback(parser)? {
                Parsed::Missing(span) => {
                    return Ok(Parsed::Missing(span));
                }

                p => p,
            };
        }

        let block = parser.assert_parsable::<Block>()?;

        Ok(Parsed::Present(Self {
            ident,
            block,
            parameters: params.unwrap_or(ParameterList::default()),
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

        if parser.maybe(TokenKind::Arrow) {
            let _path = match Path::parse_with_rollback(parser)? {
                Parsed::Missing(span) => {
                    return Ok(Parsed::Missing(span));
                }

                p => p,
            };
        }

        Ok(Parsed::Present(Self {
            ident,
            parameters: params.unwrap_or(ParameterList::default()),
        }))
    }
}

impl Parsable for ParameterList {
    const NAME: &str = "parameter list";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        let snapshot = parser.stream.snapshot();

        let tokens = match parser.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => {
                return Ok(Parsed::Missing(Span::from(
                    parser.stream.past_span_or_distance(2, snapshot).end,
                )));
            }
        };

        let mut parameters = Vec::new();

        if tokens.is_empty() {
            return Ok(Parsed::Present(Self {
                span: parser.stream.span_since(snapshot),
                parameters,
            }));
        }

        let mut inner = parser.branch(TokenStream::new(tokens));
        let mut expect_param = true;

        while expect_param {
            parameters.push(inner.assert_parsable::<Parameter>()?);

            expect_param = inner.maybe(TokenKind::Comma);
        }

        Ok(Parsed::Present(Self {
            span: parser.stream.span_since(snapshot),
            parameters,
        }))
    }
}

impl Parsable for Parameter {
    const NAME: &str = "parameter";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        let ident = parser.assert_parsable::<Ident>()?;

        parser.assert_token(TokenKind::Colon)?;

        let ty = parser.assert_parsable::<Ty>()?;

        Ok(Parsed::Present(Self { ident, ty }))
    }
}
