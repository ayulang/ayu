pub mod stream;
pub mod token;

use ariadne::{Color, Fmt, Label, Report, ReportKind};
use ayuc_common::{ARIADNE_CONFIG, SourceReport};
use ayuc_scanner::{
    Scanner,
    raw_token::{self, RawToken, RawTokenKind, RawTokenStream},
};
use ayuc_source::SourceSpan;
use ayuc_span::{Span, symbol::Symbol};
use unicode_properties::UnicodeEmoji;

use crate::token::{Delimiter, Keyword, Literal, StructuredToken, Token, TokenKind};

pub struct LexedFile<'a> {
    pub tokens: Vec<StructuredToken>,
    pub diagnostics: Vec<SourceReport<'a>>,
}

/// Lexes the whole input file and returns a [TokenStream] and the produced diagnostics.
pub fn lex(file_id: usize, source: &str) -> Result<LexedFile<'_>, Box<SourceReport<'_>>> {
    let lexer = Lexer::new(file_id, source);
    let (tokens, diagnostics) = lexer.lex_into_structured()?;

    Ok(LexedFile {
        tokens,
        diagnostics,
    })
}

pub struct Lexer<'a> {
    /// The raw tokens.
    raw_stream: RawTokenStream,

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
            raw_stream: Scanner::new(source).into(),
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
            "extern" => Some(Keyword::Extern),
            "return" => Some(Keyword::Return),
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
                let data_span = if !terminated {
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

                    return None; // maybe we change this in the future (recovery), but not yet.
                } else {
                    (span.start + 1, span.end - 1) // removes the quotes "..."
                };

                Some(TokenKind::Literal(Literal::Str {
                    data_span: data_span.into(),
                }))
            }
            raw_token::LiteralKind::Integer { data_span } => {
                Some(TokenKind::Literal(Literal::Integer { data_span }))
            }
        }
    }

    /// Lexes a vector of [StructuredToken]s until the delimiter is closed. An error means that a delimiter is not properly closed and is an unrecoverable error.
    pub(crate) fn lex_structured_until_delimiter(
        &mut self,
        span: Span,
        delimiter: Delimiter,
    ) -> Result<StructuredToken, Box<SourceReport<'a>>> {
        let mut buf = Vec::new();
        let mut full_span = span;
        let closing_kind = delimiter.closing_kind();

        loop {
            let token = self.next_token();

            match token.kind {
                kind if kind == closing_kind => {
                    full_span.end = token.span.end;

                    break;
                }

                TokenKind::OpenParen | TokenKind::OpenBrace => {
                    let delimiter = token
                        .kind
                        .to_delimiter()
                        .expect("failed to convert token kind to delimiter");

                    buf.push(self.lex_structured_until_delimiter(token.span, delimiter)?)
                }

                TokenKind::Eof => {
                    let pair = match delimiter.closing_kind() {
                        TokenKind::CloseBrace => "}",
                        TokenKind::CloseParen => "}",
                        _ => "",
                    };

                    let main_span = self.sourced_span(span);

                    return Err(Box::new(
                        Report::build(ReportKind::Error, main_span)
                            .with_config(ARIADNE_CONFIG)
                            .with_message("unclosed delimiter")
                            .with_label(
                                Label::new(main_span)
                                    .with_color(Color::BrightRed)
                                    .with_message(
                                        "delimiter starts here and is never closed"
                                            .fg(Color::BrightRed),
                                    ),
                            )
                            .with_help(format!(
                                "consider adding a `{pair}` to close this delimiter where practical"
                            ))
                            .finish(),
                    ));
                }

                _ => buf.push(StructuredToken::Token(token)),
            }
        }

        Ok(StructuredToken::Delimited(full_span, delimiter, buf))
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            let Some(RawToken {
                kind: raw_kind,
                mut span,
            }) = self.raw_stream.consume()
            else {
                todo!("have to add unexpected EOF to Lexer::next_token")
            };

            let kind = match raw_kind {
                RawTokenKind::Whitespace => {
                    continue;
                }

                RawTokenKind::Ident => self.ident_or_keyword(&span),
                RawTokenKind::Literal { kind } => {
                    if let Some(lit) = self.literal(&span, kind) {
                        lit
                    } else {
                        continue;
                    }
                }

                RawTokenKind::Plus => TokenKind::Plus,
                RawTokenKind::Minus => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Gt,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::Arrow
                    }
                    _ => TokenKind::Minus,
                },
                RawTokenKind::Semi => TokenKind::Semi,
                RawTokenKind::Colon => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Colon,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::DoubleColon
                    }
                    _ => TokenKind::Colon,
                },
                RawTokenKind::Equals => TokenKind::Equals,
                RawTokenKind::OpenParen => TokenKind::OpenParen,
                RawTokenKind::CloseParen => TokenKind::CloseParen,
                RawTokenKind::OpenBrace => TokenKind::OpenBrace,
                RawTokenKind::CloseBrace => TokenKind::CloseBrace,
                RawTokenKind::Gt => TokenKind::Gt,
                RawTokenKind::Comma => TokenKind::Comma,

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

    pub fn lex_into_structured(
        mut self,
    ) -> Result<(Vec<StructuredToken>, Vec<SourceReport<'a>>), Box<SourceReport<'a>>> {
        let mut buf = Vec::new();

        loop {
            let token = self.next_token();

            match token.kind {
                TokenKind::CloseParen | TokenKind::CloseBrace => {
                    let pair = match token.kind {
                        TokenKind::CloseBrace => "{",
                        TokenKind::CloseParen => "(",
                        _ => "",
                    };

                    let main_span = self.sourced_span(token.span);
                    let src = &self.source[token.span];

                    return Err(Box::new(
                        Report::build(ReportKind::Error, main_span)
                            .with_config(ARIADNE_CONFIG)
                            .with_message(format!("unexpected closing delimiter: `{src}`"))
                            .with_label(
                                Label::new(main_span)
                                    .with_color(Color::BrightRed)
                                    .with_message(
                                        "unexpected closing delimiter".fg(Color::BrightRed),
                                    ),
                            )
                            .with_note(format!("this delimiter needs a matching `{pair}`"))
                            .finish(),
                    ));
                }

                TokenKind::OpenParen | TokenKind::OpenBrace => {
                    let delimiter = token
                        .kind
                        .to_delimiter()
                        .expect("failed to convert token kind to delimiter");

                    buf.push(self.lex_structured_until_delimiter(token.span, delimiter)?)
                }

                TokenKind::Eof => {
                    buf.push(StructuredToken::Token(token));

                    break;
                }

                _ => {
                    buf.push(StructuredToken::Token(token));
                }
            }
        }

        Ok((buf, self.diagnostics))
    }
}
