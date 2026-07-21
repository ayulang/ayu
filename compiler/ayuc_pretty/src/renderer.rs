use crate::{
    config::{Config, Mode},
    doc::Doc,
};

pub struct Renderer {
    config: Config,

    current_indentation: usize,

    full_indent_str: String,
    single_indent_str: String,
}

impl Renderer {
    pub fn new() -> Self {
        let config = Config::default();
        let single_indent_str = " ".repeat(config.indent_level);

        Self {
            config,
            current_indentation: 0,
            single_indent_str,
            full_indent_str: String::new(),
        }
    }

    #[inline]
    pub fn with_config(mut self, config: Config) -> Self {
        self.single_indent_str = " ".repeat(config.indent_level);
        self.config = config;

        self.compute_indentation();

        self
    }

    pub fn render(&mut self, doc: &Doc) -> String {
        let mut buf = String::new();

        self.render_to(&mut buf, doc);

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
                    buf.push_str(&self.full_indent_str);
                }
            },
            Doc::Indent(doc) => match self.config.mode {
                Mode::Pretty => {
                    self.bump_indentation();

                    let text = self.render(doc);

                    if !text.is_empty() {
                        buf.push_str(&self.single_indent_str);
                        buf.push_str(&text);
                    }

                    self.pop_indentation();
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

                buf.push_str(&self.full_indent_str);
            }
            Doc::Separated(docs, sep) => {
                for (i, doc) in docs.iter().enumerate() {
                    if i != 0 {
                        self.render_to(buf, sep);
                    }

                    self.render_to(buf, doc);
                }
            }
            Doc::Skip => {}
        }
    }

    fn bump_indentation(&mut self) {
        self.current_indentation += 1;
        self.compute_indentation();
    }

    fn pop_indentation(&mut self) {
        self.current_indentation -= 1;
        self.compute_indentation();
    }

    fn compute_indentation(&mut self) {
        self.full_indent_str = self.single_indent_str.repeat(self.current_indentation);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
