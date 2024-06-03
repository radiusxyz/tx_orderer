use axum::{http::StatusCode, response::IntoResponse};

pub enum Error {
    Boxed(Box<dyn std::error::Error>),
    SignatureMismatch,
    HealthCheck,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::SignatureMismatch => write!(f, "Sender is not the signer."),
            Self::HealthCheck => {
                write!(
                    f,
                    "Health-check failed. Make sure the sequencer is running and port-forwarded."
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Boxed(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            Self::SignatureMismatch => (StatusCode::INTERNAL_SERVER_ERROR, self).into_response(),
            Self::HealthCheck => (StatusCode::INTERNAL_SERVER_ERROR, self).into_response(),
        }
    }
}

impl Error {
    pub fn boxed<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::Boxed(Box::new(error))
    }
}
