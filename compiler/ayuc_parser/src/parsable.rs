use ariadne::{Color, Fmt, Label, Report};
use ayuc_source::SourceSpan;

use crate::Parser;

pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

pub trait Parsable: Sized {
    /// Parses this node. This is used for "expecting" nodes.
    fn parse<'a>(input: &mut Parser<'a>) -> ParseResult<'a, Self>;
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

    pub fn with_expected_report(self, span: SourceSpan, expected: &str, got: &str) -> Self {
        let got = if got.len() > 16 {
            &format!("{}...{}", &got[..8], &got[got.len() - 8..])
        } else {
            got
        };

        self.with_report(
            Report::build(ariadne::ReportKind::Error, span)
                .with_message(format!("expected {expected}, got `{got}`"))
                .with_label(
                    Label::new(span)
                        .with_color(Color::BrightRed)
                        .with_message(format!("expected {expected}").fg(Color::BrightRed)),
                )
                .finish(),
        )
    }
}
