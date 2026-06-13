use std::rc::Rc;

use ariadne::Source;

pub type FileId = usize;

#[derive(Default)]
pub struct SourceCache {
    /// A vector of file names and source texts.
    files: Vec<(Rc<String>, Source)>,
}

impl SourceCache {
    pub fn add<N: Into<String>, S: Into<String>>(&mut self, name: N, source: S) -> FileId {
        self.files
            .push((Rc::new(name.into()), Source::from(source.into())));

        self.files.len() - 1
    }

    pub fn source_of(&self, file_id: FileId) -> Option<&Source> {
        self.files.get(file_id).map(|(_, source)| source)
    }

    pub fn name_of(&self, file_id: FileId) -> Option<Rc<String>> {
        self.files.get(file_id).map(|(name, _)| name).cloned()
    }
}

impl ariadne::Cache<FileId> for &SourceCache {
    type Storage = String;

    fn display<'a>(&self, id: &'a FileId) -> Option<impl std::fmt::Display + 'a> {
        self.name_of(*id)
    }

    fn fetch(&mut self, id: &FileId) -> Result<&Source<Self::Storage>, impl std::fmt::Debug> {
        self.source_of(*id)
            .ok_or_else(|| format!("source with id {id} is not present in source list"))
    }
}
