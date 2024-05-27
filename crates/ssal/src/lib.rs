pub mod ethereum;
pub mod event;

use primitives::{async_trait::async_trait, error::Error};

#[async_trait]
pub trait Client {
    async fn initialize_cluster(&self) -> Result<(), Error>;

    async fn register_sequencer(&self) -> Result<(), Error>;

    async fn deregister_sequencer(&self) -> Result<(), Error>;
}
