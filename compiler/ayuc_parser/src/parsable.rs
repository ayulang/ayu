use ariadne::Report;
use ayuc_lexer::stream::TokenStream;
use ayuc_source::SourceSpan;

pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

pub trait Parsable: Sized {
    /// The name of the parsable node. Used for error diagnostics.
    const NAME: &str;

    /// Tries to parse this node.
    fn try_parse(input: &mut TokenStream) -> ParseResult<'_, Self>;
}

pub struct ParseError<'a> {
    pub report: Option<Report<'a, SourceSpan>>,
}

impl<'a> ParseError<'a> {
    pub const fn new() -> Self {
        Self { report: None }
    }

    pub fn with_report(mut self, report: Report<'a, SourceSpan>) -> Self {
        self.report = Some(report);

        self
    }
}
