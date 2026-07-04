pub enum Mode {
    Pretty,
    OneLine,
}

pub struct Config {
    pub mode: Mode,
    pub indent_level: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Pretty,
            indent_level: 4,
        }
    }
}
