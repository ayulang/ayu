pub mod stream;
pub mod token;

use ariadne::{Color, Config, Fmt, IndexType, Label, Report, ReportKind};
use ayuc_scanner::{
    Scanner,
    raw_token::{self, RawToken, RawTokenKind},
};
use ayuc_source::SourceSpan;
use ayuc_span::{Span, symbol::Symbol};
use unicode_properties::UnicodeEmoji;

use crate::{
    stream::TokenStream,
    token::{Keyword, Token, TokenKind},
};

const ARIADNE_CONFIG: Config = Config::new().with_index_type(IndexType::Byte);

/// Lexes the whole input file and returns a [TokenStream] and the produced diagnostics.
pub fn lex(file_id: usize, source: &str) -> (TokenStream, Vec<Report<'_, SourceSpan>>) {
    let mut lexer = Lexer::new(file_id, source);
    let mut buf = Vec::new();

    loop {
        let token = lexer.next_token();
        let is_eof = token.is_eof();

        buf.push(token);

        if is_eof {
            break;
        }
    }

    (TokenStream::new(buf), lexer.diagnostics)
}

pub struct Lexer<'a> {
    /// The token scanner.
    scanner: Scanner<'a>,

    /// The source string.
    source: &'a str,
    /// The ID of the source file. Used for diagnostics.
    file_id: usize,

    /// The produced diagnostics.
    pub diagnostics: Vec<Report<'a, SourceSpan>>,
}

impl<'a> Lexer<'a> {
    pub fn new(file_id: usize, source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            source,
            file_id,
            diagnostics: Vec::new(),
        }
    }

    /// Transforms the input span into a span with the current source file's id for error diagnostics.
    pub(crate) fn sourced_span<S: Into<Span>>(&self, span: S) -> SourceSpan {
        SourceSpan::new(self.file_id, span.into())
    }

    pub(crate) fn ident_or_keyword(&mut self, span: &Span) -> TokenKind {
        let ident = &self.source[span];
        let keyword = match ident {
            "fn" => Some(Keyword::Fn),
            "let" => Some(Keyword::Let),
            _ => None,
        };

        keyword
            .map(TokenKind::Keyword)
            .unwrap_or_else(|| TokenKind::Ident(Symbol::intern(ident)))
    }

    pub(crate) fn literal(
        &mut self,
        span: &Span,
        kind: raw_token::LiteralKind,
    ) -> Option<TokenKind> {
        match kind {
            raw_token::LiteralKind::Str { terminated } => {
                if !terminated {
                    let sourced_span = self.sourced_span(span);
                    let report = Report::build(ReportKind::Error, sourced_span)
                        .with_config(ARIADNE_CONFIG)
                        .with_message("unterminated double-quote string")
                        .with_label(
                            Label::new(self.sourced_span(Span::from(span.start..span.start + 1)))
                                .with_color(Color::BrightBlue)
                                .with_message("string starts here".fg(Color::BrightBlue)),
                        )
                        .with_label(
                            Label::new(sourced_span)
                                .with_message("string has no end".fg(Color::BrightRed))
                                .with_color(Color::BrightRed),
                        )
                        .with_help("consider adding a `\"` to terminate the string");

                    self.diagnostics.push(report.finish());

                    return None;
                }

                let data_span = (span.start + 1, span.end - 1);

                Some(TokenKind::Literal {
                    data_span: data_span.into(),
                })
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            let RawToken {
                kind: raw_kind,
                span,
            } = self.scanner.next_token();

            let kind = match raw_kind {
                RawTokenKind::Whitespace => continue,

                RawTokenKind::Ident => self.ident_or_keyword(&span),
                RawTokenKind::Literal { kind } => {
                    if let Some(lit) = self.literal(&span, kind) {
                        lit
                    } else {
                        continue;
                    }
                }

                RawTokenKind::Semi => TokenKind::Semi,
                RawTokenKind::Colon => TokenKind::Colon,
                RawTokenKind::Equals => TokenKind::Equals,
                RawTokenKind::OpenParen => TokenKind::OpenParen,
                RawTokenKind::CloseParen => TokenKind::CloseParen,
                RawTokenKind::OpenBrace => TokenKind::OpenBrace,
                RawTokenKind::CloseBrace => TokenKind::CloseBrace,

                // Provide a diagnostic about the invalid identifier. We return [TokenKind::Ident] anyway, so the parser can continue.
                RawTokenKind::InvalidIdent => {
                    let emoji_props = self.source[span]
                        .chars()
                        .enumerate()
                        .find(|c| c.1.is_emoji_char())
                        .map(|(idx, c)| (span.start + idx, c.len_utf8()));

                    let main_span = self.sourced_span(span);

                    let label = if let Some((pos, len)) = emoji_props {
                        Label::new(self.sourced_span(Span::from(pos..pos + len)))
                            .with_color(Color::BrightRed)
                            .with_message(
                                "emojis are not permitted in identifiers".fg(Color::BrightRed),
                            )
                    } else {
                        Label::new(main_span)
                            .with_color(Color::BrightRed)
                            .with_message("this is an invalid identifier".fg(Color::BrightRed))
                    };

                    let report = Report::build(ReportKind::Error, main_span)
                        .with_config(ARIADNE_CONFIG)
                        .with_message("invalid identifier")
                        .with_label(label);

                    self.diagnostics.push(report.finish());

                    TokenKind::Ident(Symbol::intern(&self.source[span]))
                }

                RawTokenKind::Unknown => {
                    let sourced_snap = self.sourced_span(span);

                    let report = Report::build(ReportKind::Error, sourced_snap)
                        .with_config(ARIADNE_CONFIG)
                        .with_message("unknown token")
                        .with_label(
                            Label::new(sourced_snap)
                                .with_color(Color::BrightRed)
                                .with_message("unknown start of token".fg(Color::BrightRed)),
                        );

                    self.diagnostics.push(report.finish());

                    continue;
                }

                RawTokenKind::Eof => TokenKind::Eof,
            };

            return Token::new(kind, span);
        }
    }
}
