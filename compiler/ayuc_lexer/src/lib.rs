use ayuc_scanner::Scanner;

pub struct Lexer<'a> {
    /// The source string. Reserved for error diagnostics.
    source: &'a str,
    /// The token scanner.
    scanner: Scanner<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            scanner: Scanner::new(source),
        }
    }
}
