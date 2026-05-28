pub mod predicate;
pub mod raw_token;

use std::str::Chars;

use unicode_properties::UnicodeEmoji;

use crate::raw_token::{RawToken, RawTokenKind};

/// Transforms the input source code to a stream of [RawToken]s.
/// These are basically normal tokens that only contain the token kind and location, no additional data.
pub struct Scanner<'a> {
    source: &'a str,
    chars: Chars<'a>,
    position: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            position: 0,
        }
    }

    pub(crate) fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(current) = self.first()
            && predicate(current)
        {
            self.position += 1;

            self.chars.next();
        }
    }

    pub(crate) fn whitespace(&mut self) -> RawToken {
        let start = self.position - 1;

        self.eat_while(|c| c.is_whitespace());

        RawToken::new(RawTokenKind::Whitespace, (start, self.position))
    }

    pub(crate) fn ident(&mut self) -> RawToken {
        let start = self.position - 1;

        self.eat_while(predicate::is_ident_continue);

        match self.first() {
            Some(c) if !c.is_ascii() && c.is_emoji_char() => self.invalid_ident(start),
            _ => RawToken::new(RawTokenKind::Ident, (start, self.position)),
        }
    }

    pub(crate) fn invalid_ident(&mut self, ident_start: usize) -> RawToken {
        self.eat_while(|c| predicate::is_ident_continue(c) || (!c.is_ascii() && c.is_emoji_char()));

        RawToken::new(RawTokenKind::InvalidIdent, (ident_start, self.position))
    }

    pub fn next_token(&mut self) -> RawToken {
        let Some(first_char) = self.chars.next() else {
            return RawToken::new(RawTokenKind::Eof, self.source.len());
        };

        self.position += 1;

        match first_char {
            c if c.is_whitespace() => self.whitespace(),
            c if predicate::is_ident_start(c) => self.ident(),
            _ => RawToken::new(RawTokenKind::Unknown, (self.position - 1, self.position)),
        }
    }

    pub fn first(&self) -> Option<char> {
        self.chars.clone().next()
    }

    pub fn second(&self) -> Option<char> {
        let mut chars = self.chars.clone();

        chars.next();

        chars.next()
    }

    pub fn third(&self) -> Option<char> {
        let mut chars = self.chars.clone();

        chars.next();
        chars.next();

        chars.next()
    }
}
