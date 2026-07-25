use ayuc_diagnostic::DiagnosticContext;
use ayuc_registry::ModuleRegistry;
use ayuc_session::Session;
use ayuc_source::SourceCache;

#[derive(Default)]
pub struct CompilerContext {
    pub sess: Session,
    pub source_cache: SourceCache,
    pub module_registry: ModuleRegistry,
    pub dcx: DiagnosticContext,
}
