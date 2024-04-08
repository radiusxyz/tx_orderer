use sequencer_database::Database;

pub fn database() -> Database {
    crate::sequencer::sequencer().database()
}

pub fn spawn<F>(future: F) -> sequencer_core::tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    crate::sequencer::sequencer().spawn(future)
}
