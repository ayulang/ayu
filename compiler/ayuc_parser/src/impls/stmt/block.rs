use ayuc_ir::node::stmt::block::Block;
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken},
};
use ayuc_span::Span;

use crate::{
    Parser,
    parsable::{Assertable, Parsable, Parsed},
};

impl Parsable for Block {
    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ()> {
        let snapshot = parser.stream.snapshot();

        let (span, tokens) = match parser.stream.consume() {
            Some(StructuredToken::Delimited(span, Delimiter::Braces, tokens)) => (*span, tokens),
            _ => {
                return Ok(Parsed::Missing(Span::from(
                    parser.stream.past_span_or_distance(2, snapshot).end,
                )));
            }
        };

        let inner = parser.branch(TokenStream::new(tokens));
        let (ast, session) = inner.parse_full();

        session.commit(&mut parser.session);

        if let Some(ast) = ast {
            Ok(Parsed::Present(Self {
                span,
                children: ast.nodes,
            }))
        } else {
            Err(()) // unrecoverable error
        }
    }
}

impl Assertable for Block {
    const NAME: &str = "block";
}
