use ayuc_source::cache::FileId;
use ayuc_span::Span;

const ARIADNE_CONFIG: ariadne::Config =
    ariadne::Config::new().with_index_type(ariadne::IndexType::Byte);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Advice,
}

pub struct Diagnostic {
    pub file_id: FileId,
    pub severity: Severity,
    pub span: Span,
    pub message: Option<String>,
}

pub struct DiagnosticContext {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostic {
    pub fn new(file_id: FileId, severity: Severity, span: Span) -> Self {
        Self {
            file_id,
            severity,
            span,
            message: None,
        }
    }

    #[inline]
    pub fn with_message<M>(mut self, message: M) -> Self
    where
        M: AsRef<str>,
    {
        self.message = Some(message.as_ref().to_string());

        self
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
    pub fn diagnostics_by_severity(&self, severity: Severity) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .collect::<Vec<_>>()
    }

    #[inline]
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics_by_severity(Severity::Error)
    }

    #[inline]
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics_by_severity(Severity::Warning)
    }

    #[inline]
    pub fn advice(&self) -> Vec<&Diagnostic> {
        self.diagnostics_by_severity(Severity::Advice)
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
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Advice => Self::Advice,
        }
    }
}
