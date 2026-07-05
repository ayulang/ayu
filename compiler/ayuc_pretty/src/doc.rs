pub enum Doc {
    /// A text.
    Text(String),
    /// Concatenates a list of docs together.
    Concat(Vec<Doc>),
    /// An optional newline for beauty purposes.
    Blankline,
    /// Newline in pretty mode, whitespace in one-line mode.
    Hardline,
    /// Used for explicitly separating statements. The mode decides what character it is.
    StmtSep,
    /// Indents the following [Doc].
    Indent(Box<Doc>),
}

impl Doc {
    pub fn text<S: AsRef<str>>(text: S) -> Self {
        Self::Text(text.as_ref().to_string())
    }

    pub fn indent(doc: Doc) -> Self {
        Self::Indent(Box::new(doc))
    }
}
