use ayuc_ast::{Path, Ty, TyKind};
use ayuc_span::Span;

use crate::{
    Parser,
    parsable::{Parsable, ParseError, Parsed},
};

impl Parsable for Ty {
    const NAME: &str = "type";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        let id = parser.node_id_allocator.allocate();
        let snapshot = parser.stream.snapshot();

        if let Parsed::Present(path) = Path::parse_with_rollback(parser)? {
            Ok(Parsed::Present(Ty {
                id,
                span: parser.stream.span_since(snapshot),
                kind: TyKind::Path(path),
            }))
        } else {
            Ok(Parsed::Missing(Span::from(
                parser.stream.past_span_or_distance(1, snapshot).end,
            )))
        }
    }
}
