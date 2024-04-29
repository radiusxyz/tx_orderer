// use std::future::Future;

// use sequencer_core::tokio::task::JoinHandle;
// use sequencer_database::Database;

// pub fn database() -> Database {
//     crate::sequencer::sequencer().database()
// }

// pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
// where
//     F: Future + Send + 'static,
//     F::Output: Send + 'static,
// {
//     crate::sequencer::sequencer().spawn(future)
// }

// pub fn block_on<F>(future: F) -> F::Output
// where
//     F: Future,
// {
//     crate::sequencer::sequencer().block_on(future)
// }
