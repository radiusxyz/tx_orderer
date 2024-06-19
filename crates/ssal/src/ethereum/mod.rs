mod client;
mod error;
mod event;
pub mod seeder;
pub mod types;

pub use client::SsalClient;
pub use error::{Error, ErrorKind};
pub use event::SsalEventListener;
