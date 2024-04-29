use std::fmt::{Arguments, Write};

pub struct Context(String);

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.is_empty() {
            true => Ok(()),
            false => writeln!(f, "Context: {}", self.0),
        }
    }
}

impl std::fmt::Write for Context {
    fn write_fmt(&mut self, args: Arguments<'_>) -> std::fmt::Result {
        self.0.write_fmt(args)
    }

    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_str(s)
    }
}

impl From<Arguments<'_>> for Context {
    fn from(value: Arguments<'_>) -> Self {
        let mut context = Self::empty();
        context.write_fmt(value).unwrap();
        context
    }
}

impl From<&str> for Context {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Context {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Context {
    pub fn empty() -> Self {
        Self(String::new())
    }

    pub fn as_string(self) -> String {
        self.0
    }
}
