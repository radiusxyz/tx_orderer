use std::panic::Location;

pub struct Context {
    location: std::panic::Location<'static>,
    context: String,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.context,
            self.location.file(),
            self.location.line(),
        )
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl Context {
    #[track_caller]
    pub fn new<C>(context: C) -> Self
    where
        C: std::fmt::Debug,
    {
        Self {
            location: *Location::caller(),
            context: format!("{:?}", context),
        }
    }
}
