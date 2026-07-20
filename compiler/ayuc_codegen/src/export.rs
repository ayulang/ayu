#[derive(Debug)]
pub enum Export<'a> {
    Mapped {
        name: &'a str,
        absolute: String,
    },
    Module {
        name: &'a str,
        children: Vec<Export<'a>>,
    },
}

impl<'a> Export<'a> {
    pub fn mapped(name: &'a str, absolute: String) -> Self {
        Self::Mapped { name, absolute }
    }

    pub fn module(name: &'a str, children: Vec<Export<'a>>) -> Self {
        Self::Module { name, children }
    }
}
