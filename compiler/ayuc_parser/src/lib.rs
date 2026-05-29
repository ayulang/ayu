pub mod parsable;

use ariadne::Report;
use ayuc_common::SourceReport;
use ayuc_ir::Ast;
use ayuc_lexer::stream::TokenStream;
use ayuc_source::SourceSpan;

/// Used for parsing an input file into an abstract syntax tree.
pub struct Parser<'a> {
    /// The input token stream.
    stream: TokenStream,
    /// The produced diagnostics from both the parser and lexer.
    diagnostics: Vec<Report<'a, SourceSpan>>,
}

impl<'a> Parser<'a> {
    /// This function only errors if an unrecoverable error occurred while lexing the input.
    pub fn new(file_id: usize, source: &'a str) -> Result<Self, SourceReport<'a>> {
        let (stream, diagnostics) = ayuc_lexer::lex(file_id, source)?;

        Ok(Self {
            stream,
            diagnostics,
        })
    }

    pub fn parse_full(&mut self) -> Ast {
        todo!()
    }
}
