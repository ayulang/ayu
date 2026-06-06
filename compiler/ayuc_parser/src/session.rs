use ayuc_common::SourceReport;

#[derive(Default)]
pub struct ParseSession<'a>(Vec<SourceReport<'a>>);

impl<'a> ParseSession<'a> {
    pub fn new(reports: Vec<SourceReport<'a>>) -> Self {
        Self(reports)
    }

    pub fn emit(&mut self, report: SourceReport<'a>) {
        self.0.push(report);
    }

    pub fn unwrap(self) -> Vec<SourceReport<'a>> {
        self.0
    }

    pub fn commit(self, other: &mut ParseSession<'a>) {
        other.0.extend(self.0);
    }

    pub fn extend(&mut self, reports: Vec<SourceReport<'a>>) {
        self.0.extend(reports);
    }

    pub fn discard(self) {}
}
