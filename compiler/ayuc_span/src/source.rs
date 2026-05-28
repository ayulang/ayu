pub struct SourceFile<'a> {
    pub name: &'a str,
    pub data: &'a str,
}

impl<'a> SourceFile<'a> {
    pub const fn new(name: &'a str, data: &'a str) -> Self {
        Self { name, data }
    }

    pub const fn from_memory(data: &'a str) -> Self {
        Self::new("<memory>", data)
    }

    #[inline]
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = name;

        self
    }
}

impl<'a> From<&'a str> for SourceFile<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            name: "<unknown>",
            data: value,
        }
    }
}
