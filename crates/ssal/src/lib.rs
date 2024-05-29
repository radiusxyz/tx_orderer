pub mod ethereum;
pub mod types;

use primitives::{async_trait::async_trait, error::Error};

#[async_trait]
pub trait Client {
    // async fn initialize_cluster(&self) -> Result<(), Error>;

    // async fn register_sequencer(&self) -> Result<(), Error>;

    // async fn deregister_sequencer(&self) -> Result<(), Error>;
    async fn get_block(&self) -> Result<usize, Error>;

    async fn get_sequencer_list(&self) -> Result<(), Error>;
}
