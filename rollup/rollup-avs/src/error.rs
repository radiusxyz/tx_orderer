pub type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
pub enum ErrorKind {
    SequencerNotAvailable,
    ClusterDown,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ErrorKind {}
