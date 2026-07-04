use crate::{
    config::{Config, Mode},
    doc::Doc,
};

pub struct Renderer {
    config: Config,
    current_indentation: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            current_indentation: 0,
        }
    }

    #[inline]
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;

        self
    }

    pub fn render(mut self, doc: Doc) -> String {
        let mut buf = String::new();

        self.render_to(&mut buf, &doc);

        buf
    }

    fn render_to(&mut self, buf: &mut String, doc: &Doc) {
        match doc {
            Doc::Blankline => match self.config.mode {
                Mode::OneLine => {}
                Mode::Pretty => buf.push('\n'),
            },
            Doc::Hardline => match self.config.mode {
                Mode::OneLine => {
                    buf.push(' ');
                }
                Mode::Pretty => {
                    buf.push('\n');
                    buf.push_str(&self.full_indent());
                }
            },
            Doc::Indent(doc) => match self.config.mode {
                Mode::Pretty => {
                    self.current_indentation += 1;

                    buf.push_str(&self.another_indent());

                    self.render_to(buf, doc);

                    self.current_indentation -= 1;
                }
                Mode::OneLine => self.render_to(buf, doc),
            },
            Doc::Text(t) => buf.push_str(t),
            Doc::Concat(docs) => {
                for doc in docs {
                    self.render_to(buf, doc);
                }
            }
            Doc::StmtSep => {
                buf.push(match self.config.mode {
                    Mode::Pretty => '\n',
                    Mode::OneLine => ';',
                });

                buf.push_str(&self.full_indent());
            }
        }
    }

    fn full_indent(&self) -> String {
        " ".repeat(self.current_indentation * self.config.indent_level)
    }

    fn another_indent(&self) -> String {
        " ".repeat(self.config.indent_level)
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
