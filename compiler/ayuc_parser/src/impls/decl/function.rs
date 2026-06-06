use ariadne::{Color, Fmt, Label};
use ayuc_common::{ARIADNE_CONFIG, SourceReport};
use ayuc_ir::node::{
    decl::function::{FnDecl, ParameterList},
    leaf::ident::Ident,
    stmt::block::Block,
};
use ayuc_lexer::token::{Delimiter, Keyword, StructuredToken};
use ayuc_span::Span;

use crate::{
    Parser,
    parsable::{Parsable, Parsed},
};

impl Parsable for FnDecl {
    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ()> {
        parser.assert_keyword(Keyword::Fn)?;

        let ident = parser.assert_parsable::<Ident>()?;

        let params = match ParameterList::parse(parser)? {
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

        Ok(Parsed::Present(FnDecl {
            ident: ident,
            parameters: params.unwrap_or(ParameterList::default()),
            block,
        }))
    }
}

impl Parsable for ParameterList {
    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ()> {
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
