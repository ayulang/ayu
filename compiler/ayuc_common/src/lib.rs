use ariadne::{IndexType, Report};
use ayuc_source::SourceSpan;

pub static ARIADNE_CONFIG: ariadne::Config =
    ariadne::Config::new().with_index_type(IndexType::Byte);

pub type SourceReport<'a> = Report<'a, SourceSpan>;
