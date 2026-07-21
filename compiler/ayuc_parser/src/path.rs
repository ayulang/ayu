use ayuc_ast::{Path, PathSegment};
use ayuc_lexer::token::{StructuredToken, Token, TokenKind};

use crate::{PResult, Parser};

impl Parser<'_, '_, '_> {
    pub fn parse_path(&mut self) -> PResult<Path> {
        let snapshot = self.stream.snapshot();

        let mut segments = Vec::new();
        let mut expect_segment = true;

        while expect_segment {
            let ident = self.parse_ident()?;

            segments.push(PathSegment {
                id: self.node_id_allocator.allocate(),
                ident,
            });

            if let Some(StructuredToken::Token(Token {
                kind: TokenKind::DoubleColon,
                ..
            })) = self.stream.first()
            {
                self.stream.consume();

                expect_segment = true;
            } else {
                expect_segment = false;
            }
        }

        Ok(Path {
            id: self.node_id_allocator.allocate(),
            span: self.stream.span_since(snapshot),
            segments,
        })
    }
}
