#[derive(Debug, Clone)]
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
    /// Not printed.
    Skip,
}

impl Doc {
    pub fn text<S: AsRef<str>>(text: S) -> Self {
        Self::Text(text.as_ref().to_string())
    }

    pub fn concat(docs: impl Into<Vec<Self>>) -> Self {
        Self::Concat(docs.into())
    }

    pub fn indent(doc: Doc) -> Self {
        Self::Indent(Box::new(doc))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Skip => true,
            Self::Text(s) => s.is_empty(),
            Self::Concat(children) => children.is_empty() || children.iter().all(|d| d.is_empty()),
            Self::Indent(doc) => doc.is_empty(),
            _ => false,
        }
    }
}
