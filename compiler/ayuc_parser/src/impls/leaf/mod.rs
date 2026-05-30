use ayuc_ir::node::leaf::ident::Ident;
use ayuc_lexer::token::{StructuredToken, Token, TokenKind};

use crate::{
    Parser,
    parsable::{Parsable, ParseError, ParseResult},
};

impl Parsable for Ident {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        let next = parser.expect_token()?;

        if let StructuredToken::Token(Token {
            kind: TokenKind::Ident(ident),
            span,
        }) = next
        {
            Ok(Self { sym: ident, span })
        } else {
            let span = next.span();

            Err(ParseError::new().with_expected_report(
                parser.sourced_span(span),
                "identifier",
                &parser.source[span],
            ))
        }
    }
}
