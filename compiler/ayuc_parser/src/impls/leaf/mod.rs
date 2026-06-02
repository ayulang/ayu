use ayuc_ir::node::leaf::ident::Ident;
use ayuc_lexer::token::{StructuredToken, Token, TokenKind};

use crate::{
    Parser,
    parsable::{Parsable, Parsed},
    session::ParseSession,
};

impl Parsable for Ident {
    fn parse<'a>(
        parser: &mut Parser<'a>,
        _sess: &mut ParseSession<'a>,
    ) -> Result<Parsed<Self>, ()> {
        let snapshot = parser.stream.snapshot();

        if let Some(StructuredToken::Token(Token {
            kind: TokenKind::Ident(sym),
            span,
        })) = parser.stream.consume()
        {
            Ok(Parsed::Present(Self {
                sym: *sym,
                span: *span,
            }))
        } else {
            Ok(Parsed::Missing(
                parser.stream.past_span_or_distance(1, snapshot),
            ))
        }
    }
}
