pub mod stream;
pub mod token;

use ayuc_diagnostic::{Diagnostic, DiagnosticContext, Label};
use ayuc_scanner::{
    Scanner,
    raw_token::{self, RawToken, RawTokenKind, RawTokenStream},
};
use ayuc_span::{Span, symbol::Symbol};
use unicode_properties::UnicodeEmoji;

use crate::token::{Delimiter, InplSegment, Keyword, Literal, StructuredToken, Token, TokenKind};

pub struct LexedFile {
    pub tokens: Vec<StructuredToken>,
}

/// Lexes the whole input file and returns a [TokenStream]. Errors only on unrecoverable errors.
pub fn lex(dcx: &mut DiagnosticContext, file_id: usize, source: &str) -> Option<LexedFile> {
    let lexer = Lexer::new(dcx, file_id, source);
    let tokens = lexer.lex_into_structured()?;

    Some(LexedFile { tokens })
}

pub struct Lexer<'a> {
    /// The raw tokens.
    raw_stream: RawTokenStream,

    /// The source string.
    source: &'a str,
    /// The ID of the source file. Used for diagnostics.
    file_id: usize,

    /// The produced diagnostics.
    dcx: &'a mut DiagnosticContext,
}

impl<'a> Lexer<'a> {
    pub fn new(dcx: &'a mut DiagnosticContext, file_id: usize, source: &'a str) -> Self {
        Self {
            raw_stream: Scanner::new(source).into(),
            source,
            file_id,
            dcx,
        }
    }

    pub(crate) fn ident_or_reserved(&mut self, span: &Span) -> TokenKind {
        let ident = &self.source[span];
        let keyword = match ident {
            "fn" => Some(Keyword::Fn),
            "let" => Some(Keyword::Let),
            "extern" => Some(Keyword::Extern),
            "return" => Some(Keyword::Return),
            "if" => Some(Keyword::If),
            "as" => Some(Keyword::As),
            "true" => return TokenKind::Literal(Literal::Bool { value: true }),
            "false" => return TokenKind::Literal(Literal::Bool { value: false }),
            "mut" => Some(Keyword::Mut),
            "break" => Some(Keyword::Break),
            "loop" => Some(Keyword::Loop),
            "while" => Some(Keyword::While),
            "else" => Some(Keyword::Else),
            "pub" => Some(Keyword::Pub),
            "mod" => Some(Keyword::Mod),
            _ => None,
        };

        keyword
            .map(TokenKind::Keyword)
            .unwrap_or_else(|| TokenKind::Ident(Symbol::intern(ident)))
    }

    pub(crate) fn literal(
        &mut self,
        span: Span,
        kind: raw_token::LiteralKind,
    ) -> Option<TokenKind> {
        match kind {
            raw_token::LiteralKind::Str { terminated } => {
                let data_span = if !terminated {
                    self.dcx.emit(
                        Diagnostic::error(self.file_id, span)
                            .with_message("unterminated double-quote string")
                            .with_label(Label::primary(span, "string has no end"))
                            .with_label(Label::note(
                                Span::from(span.start..span.start + 1),
                                "string starts here",
                            ))
                            .with_help("consider adding a \" to terminate the string"),
                    );

                    return None; // maybe we change this in the future (recovery), but not yet.
                } else {
                    (span.start + 1, span.end - 1) // removes the quotes "..."
                };

                Some(TokenKind::Literal(Literal::Str {
                    data_span: data_span.into(),
                }))
            }
            raw_token::LiteralKind::InterpolatedStr {
                terminated,
                segments,
            } => {
                if !terminated {
                    self.dcx.emit(
                        Diagnostic::error(self.file_id, span)
                            .with_message("unterminated interpolated string")
                            .with_label(Label::primary(span, "string has no end"))
                            .with_label(Label::note(
                                Span::from(span.start..span.start + 1),
                                "string starts here",
                            ))
                            .with_help("consider adding a ` to terminate the string"),
                    );

                    return None; // maybe we change this in the future (recovery), but not yet.
                }

                let segments = segments
                    .into_iter()
                    .flat_map(|seg| match seg {
                        raw_token::InplSegment::InvalidClosing(closing_span) => {
                            self.dcx.emit(
                                Diagnostic::error(self.file_id, closing_span)
                                    .with_message("unmatched `}` found")
                                    .with_label(Label::primary(
                                        closing_span,
                                        "this has no matching `{`",
                                    ))
                                    .with_help("if you intended to print `}`, you can escape it using `}}`"),
                            );

                            None
                        }
                        raw_token::InplSegment::Text { span } => Some(InplSegment::Text { span }),
                        raw_token::InplSegment::Var {
                            span: ident_span,
                            invalid,
                            terminated,
                        } => {
                            if !terminated {
                                self.dcx.emit(
                                    Diagnostic::error(self.file_id, ident_span)
                                        .with_message("unterminated interpolation segment")
                                        .with_label(Label::primary(
                                            Span::from((ident_span.start - 1, ident_span.end)),
                                            "expected a closing `}` after this",
                                        ))
                                        .with_help(format!(
                                            "replace `{{{}` with `{{{}}}`",
                                            &self.source[ident_span], &self.source[ident_span]
                                        )),
                                );
                            }

                            if invalid {
                                let emoji_props = self.source[ident_span]
                                    .chars()
                                    .enumerate()
                                    .find(|c| c.1.is_emoji_char())
                                    .map(|(idx, c)| (ident_span.start + idx, c.len_utf8()));

                                let label = if let Some((pos, len)) = emoji_props {
                                    Label::primary(
                                        Span::from(pos..pos + len),
                                        "emojis are not permitted in identifiers",
                                    )
                                } else {
                                    Label::primary(ident_span, "this is an invalid identifier")
                                };

                                self.dcx.emit(
                                    Diagnostic::error(self.file_id, ident_span)
                                        .with_message("invalid identifier")
                                        .with_label(label),
                                );
                            }

                            Some(InplSegment::Var {
                                span: ident_span
                            })
                        }
                    })
                    .collect::<Vec<_>>();

                Some(TokenKind::Literal(Literal::InterpolatedString {
                    span,
                    segments,
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
    ) -> Option<StructuredToken> {
        let mut buf = Vec::new();
        let mut full_span = span;
        let closing_kind = delimiter.closing_kind();

        loop {
            let token = match self.next_token() {
                Some(t) => t,
                None => {
                    let pair = match delimiter.closing_kind() {
                        TokenKind::CloseBrace => "}",
                        TokenKind::CloseParen => "}",
                        _ => "",
                    };

                    self.dcx.emit(
                        Diagnostic::error(self.file_id, span)
                            .with_message("unclosed delimiter")
                            .with_label(Label::primary(
                                span,
                                "delimiter starts here and is never closed",
                            ))
                            .with_help(format!(
                                "consider adding a `{pair}` to close this delimiter where practical"
                            )),
                    );

                    return None;
                }
            };

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

                _ => buf.push(StructuredToken::Token(token)),
            }
        }

        Some(StructuredToken::Delimited(full_span, delimiter, buf))
    }

    pub fn next_token(&mut self) -> Option<Token> {
        loop {
            let RawToken {
                kind: raw_kind,
                mut span,
            } = self.raw_stream.consume()?;

            let kind = match raw_kind {
                RawTokenKind::Whitespace | RawTokenKind::Comment => {
                    continue;
                }

                RawTokenKind::Ident => self.ident_or_reserved(&span),
                RawTokenKind::Literal { kind } => {
                    if let Some(lit) = self.literal(span, kind) {
                        lit
                    } else {
                        continue;
                    }
                }

                RawTokenKind::Plus => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Equals,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::PlusEquals
                    }
                    _ => TokenKind::Plus,
                },
                RawTokenKind::Minus => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Gt,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::Arrow
                    }
                    Some(RawToken {
                        kind: RawTokenKind::Equals,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::MinusEquals
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
                RawTokenKind::Equals => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Equals,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::EqualsEquals
                    }
                    _ => TokenKind::Equals,
                },
                RawTokenKind::OpenParen => TokenKind::OpenParen,
                RawTokenKind::CloseParen => TokenKind::CloseParen,
                RawTokenKind::OpenBrace => TokenKind::OpenBrace,
                RawTokenKind::CloseBrace => TokenKind::CloseBrace,
                RawTokenKind::Gt => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Equals,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::GtOrEqual
                    }
                    _ => TokenKind::Gt,
                },
                RawTokenKind::Lt => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Equals,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::LtOrEqual
                    }
                    _ => TokenKind::Lt,
                },
                RawTokenKind::Comma => TokenKind::Comma,

                RawTokenKind::Exclamation => match self.raw_stream.peek() {
                    Some(RawToken {
                        kind: RawTokenKind::Equals,
                        span: other_span,
                    }) => {
                        span.merge(*other_span);

                        self.raw_stream.consume();

                        TokenKind::NotEquals
                    }
                    _ => {
                        self.dcx.emit(
                            Diagnostic::error(self.file_id, span)
                                .with_message("exclamation tokens cannot be standalone")
                                .with_label(Label::primary(span, "expected `=` after this `!`")),
                        );

                        continue;
                    }
                },

                // Provide a diagnostic about the invalid identifier. We return [TokenKind::Ident] anyway, so the parser can continue.
                RawTokenKind::InvalidIdent => {
                    let emoji_props = self.source[span]
                        .chars()
                        .enumerate()
                        .find(|c| c.1.is_emoji_char())
                        .map(|(idx, c)| (span.start + idx, c.len_utf8()));

                    let label = if let Some((pos, len)) = emoji_props {
                        Label::primary(
                            Span::from(pos..pos + len),
                            "emojis are not permitted in identifiers",
                        )
                    } else {
                        Label::primary(span, "this is an invalid identifier")
                    };

                    self.dcx.emit(
                        Diagnostic::error(self.file_id, span)
                            .with_message("invalid identifier")
                            .with_label(label),
                    );

                    TokenKind::Ident(Symbol::intern(&self.source[span]))
                }

                RawTokenKind::Unknown => {
                    self.dcx.emit(
                        Diagnostic::error(self.file_id, span)
                            .with_message("unknown token")
                            .with_label(Label::primary(span, "unknown start of a token")),
                    );

                    continue;
                }

                RawTokenKind::Eof => return None,
            };

            return Some(Token::new(kind, span));
        }
    }

    pub fn lex_into_structured(mut self) -> Option<Vec<StructuredToken>> {
        let mut buf = Vec::new();

        while let Some(token) = self.next_token() {
            match token.kind {
                TokenKind::CloseParen | TokenKind::CloseBrace => {
                    let pair = match token.kind {
                        TokenKind::CloseBrace => "{",
                        TokenKind::CloseParen => "(",
                        _ => "",
                    };

                    let src = &self.source[token.span];

                    self.dcx.emit(
                        Diagnostic::error(self.file_id, token.span)
                            .with_message(format!("unexpected closing delimiter: `{src}`"))
                            .with_label(Label::primary(token.span, "unexpected closing delimiter"))
                            .with_help(format!("this delimiter needs a matching `{pair}`")),
                    );

                    return None;
                }

                TokenKind::OpenParen | TokenKind::OpenBrace => {
                    let delimiter = token
                        .kind
                        .to_delimiter()
                        .expect("failed to convert token kind to delimiter");

                    buf.push(self.lex_structured_until_delimiter(token.span, delimiter)?)
                }

                _ => {
                    buf.push(StructuredToken::Token(token));
                }
            }
        }

        Some(buf)
    }
}
