use ariadne::Report;
use ayuc_lexer::stream::TokenStream;
use ayuc_source::SourceSpan;

pub struct Parser<'a> {
    stream: TokenStream,
    diagnostics: Vec<Report<'a, SourceSpan>>,
}

impl<'a> Parser<'a> {
    pub fn new(file_id: usize, source: &'a str) -> Self {
        let (stream, diagnostics) = ayuc_lexer::lex(file_id, source);

        Self {
            stream,
            diagnostics,
        }
    }
}
