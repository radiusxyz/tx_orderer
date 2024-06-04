mod client;
mod error;
mod seeder;
pub mod types;

pub use client::SsalClient;
pub use error::{Error, ErrorKind};
pub use seeder::{Endpoint, SeederClient};
