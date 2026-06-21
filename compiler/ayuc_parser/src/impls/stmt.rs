use ayuc_ast::{Ident, VariableDeclaration};
use ayuc_lexer::token::{Keyword, StructuredToken, Token, TokenKind};

use crate::{
    Parser,
    parsable::{Parsable, ParseError, Parsed},
};

impl Parsable for VariableDeclaration {
    const NAME: &str = "variable declaration";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        parser.assert_keyword(Keyword::Let)?;

        let ident = parser.assert_parsable::<Ident>()?;
        let snapshot = parser.stream.snapshot();

        if !matches!(
            parser.stream.consume(),
            Some(StructuredToken::Token(Token {
                kind: TokenKind::Equals,
                ..
            }))
        ) {
            return Ok(Parsed::Missing(parser.stream.span_since(snapshot)));
        }

        let expr = parser.parse_expression()?;

        Ok(Parsed::Present(Self { ident, init: expr }))
    }
}
