pub mod token;

use std::ops::Range;

use ariadne::{Color, Config, Fmt, IndexType, Label, Report, ReportKind, Source};
use ayuc_scanner::{
    Scanner,
    raw_token::{self, RawToken, RawTokenKind},
};
use ayuc_span::{Span, source::SourceFile, symbol::Symbol};
use unicode_properties::UnicodeEmoji;

use crate::token::{Keyword, Token, TokenKind};

const ARIADNE_CONFIG: Config = Config::new().with_index_type(IndexType::Byte);

pub struct Lexer<'a> {
    /// The token scanner.
    scanner: Scanner<'a>,
    /// The source string. Reserved for error diagnostics.
    source: SourceFile<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: SourceFile<'a>) -> Self {
        Self {
            scanner: Scanner::new(source.data),
            source,
        }
    }

    /// Transforms the input span into a span with the current [SourceFile]'s name for error diagnostics.
    pub(crate) fn ariadne_span<S: Into<Range<usize>>>(&self, span: S) -> (&'a str, Range<usize>) {
        (self.source.name, span.into())
    }

    /// Gets an ariadne-compatible source definition for the current [SourceFile].
    pub(crate) fn ariadne_source(&self) -> (&'a str, Source<&'a str>) {
        (self.source.name, Source::from(self.source.data))
    }

    pub(crate) fn ident_or_keyword(&mut self, span: &Span) -> TokenKind {
        let ident = &self.source.data[span];
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
                    let _ = Report::build(ReportKind::Error, self.ariadne_span(span))
                        .with_config(ARIADNE_CONFIG)
                        .with_message("unterminated double-quote string")
                        .with_label(
                            Label::new(self.ariadne_span(span))
                                .with_message("string starts here".fg(Color::Red))
                                .with_color(Color::Red),
                        )
                        .with_help("add a `\"` to terminate the string")
                        .finish()
                        .eprint(self.ariadne_source())
                        .is_ok_and(|_| {
                            eprintln!();
                            true
                        });

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

                RawTokenKind::InvalidIdent => {
                    let emoji_props = self.source.data[span]
                        .chars()
                        .enumerate()
                        .find(|c| c.1.is_emoji_char())
                        .map(|(idx, c)| (span.start + idx, c.len_utf8()));

                    let _ = Report::build(ReportKind::Error, self.ariadne_span(span))
                        .with_config(ARIADNE_CONFIG)
                        .with_message("invalid identifier")
                        .with_labels(
                            vec![
                                Some(
                                    Label::new(self.ariadne_span(span))
                                        .with_color(Color::Red)
                                        .with_message(
                                            "this is an invalid identifier".fg(Color::Red),
                                        ),
                                ),
                                emoji_props.map(|(position, char_len)| {
                                    Label::new(self.ariadne_span(position..position + char_len))
                                        .with_color(Color::Yellow)
                                        .with_message("help: remove this emoji".fg(Color::Yellow))
                                }),
                            ]
                            .into_iter()
                            .flatten(),
                        )
                        .finish()
                        .eprint(self.ariadne_source())
                        .is_ok_and(|_| {
                            eprintln!();
                            true
                        });

                    continue;
                }

                RawTokenKind::Unknown => {
                    let _ = Report::build(ReportKind::Error, self.ariadne_span(span))
                        .with_config(ARIADNE_CONFIG)
                        .with_message("unknown token")
                        .with_label(
                            Label::new(self.ariadne_span(span))
                                .with_color(Color::Red)
                                .with_message("this is an unknown token".fg(Color::Red)),
                        )
                        .finish()
                        .eprint(self.ariadne_source())
                        .is_ok_and(|_| {
                            eprintln!();
                            true
                        });

                    continue;
                }

                RawTokenKind::Eof => TokenKind::Eof,
            };

            return Token::new(kind, span);
        }
    }
}
