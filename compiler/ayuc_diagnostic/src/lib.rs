use std::ops::Range;

use ayuc_source::FileId;
use ayuc_span::Span;

pub use colored;
use colored::{ColoredString, Colorize};

const ARIADNE_CONFIG: ariadne::Config =
    ariadne::Config::new().with_index_type(ariadne::IndexType::Byte);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recovery {
    /// The compiler has recovered from the error.
    Recovered,
    /// The compiler requires an abort to avoid corruption.
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    BrightRed,
    BrightYellow,
    BrightBlue,
    BrightCyan,
    BrightGreen,
    Red,
    Yellow,
    Blue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error(Recovery),
    Warning,
    Advice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelKind {
    Primary,
    Secondary,
    Help,
    Note,
}

#[derive(Debug)]
pub struct Label {
    pub span: Span,
    pub kind: LabelKind,
    pub message: String,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub file_id: FileId,
    pub severity: Severity,
    pub span: Span,
    pub message: Option<String>,
    pub labels: Vec<Label>,
    pub helps: Vec<String>,
}

pub struct DiagnosticContext {
    diagnostics: Vec<Diagnostic>,
}

impl Label {
    pub fn new<M: AsRef<str>>(kind: LabelKind, span: Span, message: M) -> Self {
        Self {
            span,
            kind,
            message: message.as_ref().to_string(),
        }
    }

    #[inline]
    pub fn primary<M: AsRef<str>>(span: Span, message: M) -> Self {
        Self::new(LabelKind::Primary, span, message)
    }

    #[inline]
    pub fn secondary<M: AsRef<str>>(span: Span, message: M) -> Self {
        Self::new(LabelKind::Secondary, span, message)
    }

    #[inline]
    pub fn help<M: AsRef<str>>(span: Span, message: M) -> Self {
        Self::new(LabelKind::Help, span, message)
    }

    #[inline]
    pub fn note<M: AsRef<str>>(span: Span, message: M) -> Self {
        Self::new(LabelKind::Note, span, message)
    }
}

impl Diagnostic {
    pub fn new(file_id: FileId, severity: Severity, span: Span) -> Self {
        Self {
            file_id,
            severity,
            span,
            message: None,
            labels: Vec::new(),
            helps: Vec::new(),
        }
    }

    pub fn error(file_id: FileId, span: Span, recovery: Recovery) -> Self {
        Self::new(file_id, Severity::Error(recovery), span)
    }

    pub fn warning(file_id: FileId, span: Span) -> Self {
        Self::new(file_id, Severity::Warning, span)
    }

    pub fn advice(file_id: FileId, span: Span) -> Self {
        Self::new(file_id, Severity::Advice, span)
    }

    #[inline]
    pub fn with_message<M>(mut self, message: M) -> Self
    where
        M: AsRef<str>,
    {
        self.message = Some(message.as_ref().to_string());

        self
    }

    #[inline]
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);

        self
    }

    #[inline]
    pub fn with_help<H: AsRef<str>>(mut self, help: H) -> Self {
        self.helps.push(help.as_ref().to_string());

        self
    }

    pub fn to_ariadne<'a>(&self) -> ariadne::Report<'a, (FileId, Range<usize>)> {
        let mut builder =
            ariadne::Report::build(self.severity.into(), (self.file_id, self.span.range()))
                .with_config(ARIADNE_CONFIG);

        if let Some(message) = &self.message {
            builder.set_message(message);
        }

        for label in &self.labels {
            let color = match label.kind {
                LabelKind::Primary => self.severity.color(),
                LabelKind::Secondary => self.severity.secondary_color(),
                LabelKind::Help => Color::BrightCyan,
                LabelKind::Note => Color::BrightGreen,
            };

            builder.add_label(
                ariadne::Label::new((self.file_id, label.span.range()))
                    .with_message(color.colorize(&label.message))
                    .with_color(color.into()),
            );
        }

        for help in &self.helps {
            builder.add_help(help);
        }

        builder.finish()
    }
}

impl DiagnosticContext {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn emit(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    #[inline]
    pub fn all(&self) -> &Vec<Diagnostic> {
        &self.diagnostics
    }

    #[inline]
    pub fn diagnostics_by_severity(&self, p: impl Fn(&Severity) -> bool) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| p(&d.severity))
            .collect::<Vec<_>>()
    }

    #[inline]
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics_by_severity(|sev| matches!(sev, Severity::Error { .. }))
    }

    #[inline]
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics_by_severity(|sev| *sev == Severity::Warning)
    }

    #[inline]
    pub fn advice(&self) -> Vec<&Diagnostic> {
        self.diagnostics_by_severity(|sev| *sev == Severity::Advice)
    }

    #[inline]
    pub fn requires_abort(&self) -> bool {
        self.diagnostics.iter().any(|d| match d.severity {
            Severity::Error(Recovery::Fatal) => true,
            _ => false,
        })
    }
}

impl Severity {
    pub(crate) fn color(&self) -> Color {
        match self {
            Self::Error { .. } => Color::BrightRed,
            Self::Warning => Color::BrightYellow,
            Self::Advice => Color::BrightBlue,
        }
    }

    pub(crate) fn secondary_color(&self) -> Color {
        match self {
            Self::Error { .. } => Color::Red,
            Self::Warning => Color::Yellow,
            Self::Advice => Color::Blue,
        }
    }
}

impl Color {
    pub fn colorize<S: AsRef<str>>(&self, s: S) -> ColoredString {
        let s = s.as_ref();

        match self {
            Color::BrightRed => s.bright_red(),
            Color::BrightBlue => s.bright_blue(),
            Color::BrightCyan => s.bright_cyan(),
            Color::BrightGreen => s.bright_green(),
            Color::BrightYellow => s.bright_yellow(),
            Color::Blue => s.blue(),
            Color::Red => s.red(),
            Color::Yellow => s.yellow(),
        }
    }
}

impl Default for DiagnosticContext {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Severity> for ariadne::ReportKind<'_> {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Error { .. } => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Advice => Self::Advice,
        }
    }
}

impl From<Color> for ariadne::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Blue => Self::Blue,
            Color::Yellow => Self::Yellow,
            Color::Red => Self::Red,
            Color::BrightBlue => Self::BrightBlue,
            Color::BrightCyan => Self::BrightCyan,
            Color::BrightGreen => Self::BrightGreen,
            Color::BrightRed => Self::BrightRed,
            Color::BrightYellow => Self::BrightYellow,
        }
    }
}
