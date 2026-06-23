use ayuc_ast::{Ident, Path, PathSegment};
use ayuc_lexer::token::{StructuredToken, Token, TokenKind};

use crate::{
    Parser,
    parsable::{Parsable, ParseError, Parsed},
};

impl Parsable for Path {
    const NAME: &str = "path";

    // TODO: Add diagnostic for unfinished paths, like: `std::x::`
    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        let snapshot = parser.stream.snapshot();

        let mut segments = Vec::new();
        let mut expect_segment = true;

        while expect_segment {
            let ident = match Ident::parse_with_rollback(parser)? {
                Parsed::Present(p) => p,
                Parsed::Missing(m) => {
                    return Ok(Parsed::Missing(m));
                }
            };

            segments.push(PathSegment {
                id: parser.node_id_allocator.allocate(),
                ident,
            });

            if let Some(StructuredToken::Token(Token {
                kind: TokenKind::DoubleColon,
                ..
            })) = parser.stream.first()
            {
                parser.stream.consume();

                expect_segment = true;
            }
        }

        Ok(Parsed::Present(Self {
            span: parser.stream.span_since(snapshot),
            segments,
        }))
    }
}
