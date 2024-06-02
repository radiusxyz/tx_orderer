use axum::{http::StatusCode, response::IntoResponse};

pub enum Error {
    Config(toml::de::Error),
    IO(tokio::io::Error),
    Sequencer(sequencer_framework::error::Error),
    Signature(sequencer_framework::ssal::ethereum::types::SignatureError),
    SignatureMismatch,
    HealthCheckFailed,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{}", error),
            Self::IO(error) => write!(f, "{}", error),
            Self::Sequencer(error) => write!(f, "{}", error),
            Self::Signature(error) => write!(f, "{}", error),
            Self::SignatureMismatch => write!(f, "Sender is not the signer."),
            Self::HealthCheckFailed => write!(
                f,
                "Health-check failed. Make sure the sequencer is running and port-forwarded."
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::Config(value)
    }
}

impl From<tokio::io::Error> for Error {
    fn from(value: tokio::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<sequencer_framework::error::Error> for Error {
    fn from(value: sequencer_framework::error::Error) -> Self {
        Self::Sequencer(value)
    }
}

impl From<sequencer_framework::ssal::ethereum::types::SignatureError> for Error {
    fn from(value: sequencer_framework::ssal::ethereum::types::SignatureError) -> Self {
        Self::Signature(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
