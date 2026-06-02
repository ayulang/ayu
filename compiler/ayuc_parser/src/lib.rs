/// Contains implementations of the [Parsable] trait for `ayuc_ir` nodes.
pub mod impls;
pub mod parsable;
pub mod session;

use ayuc_common::SourceReport;
use ayuc_ir::{Ast, node::stmt::fn_decl::FnDecl};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Keyword, StructuredToken, TokenKind},
};
use ayuc_source::SourceSpan;
use ayuc_span::Span;

use crate::{parsable::Parsable, session::ParseSession};

/// Used for parsing an input file into an abstract syntax tree.
pub struct Parser<'a> {
    /// The input token stream.
    pub(crate) stream: TokenStream,

    file_id: usize,
    source: &'a str,

    session: ParseSession<'a>,
}

impl<'a> Parser<'a> {
    /// This function only errors if an unrecoverable error occurred while lexing the input.
    pub fn new(file_id: usize, source: &'a str) -> Result<Self, SourceReport<'a>> {
        let (stream, diagnostics) = ayuc_lexer::lex(file_id, source)?;

        Ok(Self {
            stream,
            file_id,
            source,
            session: ParseSession::new(diagnostics),
        })
    }

    pub fn from_token_stream(file_id: usize, source: &'a str, stream: TokenStream) -> Self {
        Self {
            stream,
            file_id,
            source,
            session: ParseSession::default(),
        }
    }

    pub(crate) fn sourced_span(&self, span: Span) -> SourceSpan {
        SourceSpan::new(self.file_id, span)
    }

    pub fn parse_full(mut self) -> (Ast, Vec<SourceReport<'a>>) {
        let mut session = std::mem::take(&mut self.session);

        while let Some(token) = self.stream.first() {
            match token {
                StructuredToken::Token(tok) if tok.kind == TokenKind::Keyword(Keyword::Fn) => {
                    self.stream.consume();

                    let _ = FnDecl::parse(&mut self, &mut session);

                    break;
                }
                _ => {
                    self.stream.consume();
                }
            }
        }

        (Ast {}, session.unwrap())
    }
}
