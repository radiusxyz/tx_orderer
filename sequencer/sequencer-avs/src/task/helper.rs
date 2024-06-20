pub trait TaskExt {
    type Output: Send;

    fn map_task(self) -> Option<Self::Output>;
}

impl<T, E> TaskExt for Result<T, E>
where
    T: Send,
    E: std::error::Error,
{
    type Output = T;

    fn map_task(self) -> Option<Self::Output> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                tracing::error!("{}", error);
                None
            }
        }
    }
}
