pub mod seeder;
pub mod ssal;
pub mod types;

use primitives::{async_trait::async_trait, error::Error};

use crate::types::*;

#[async_trait]
pub trait SsalApi: Clone + Send + Sync {
    async fn get_sequencer_list(&self) -> Result<Vec<SequencerPublicKey>, Error>;
}

#[async_trait]
pub trait SeederApi: Clone + Send + Sync {
    async fn register(&self) -> Result<(), Error>;

    async fn deregister(&self) -> Result<(), Error>;

    async fn get_address_list(
        &self,
        sequencer_list: Vec<SequencerPublicKey>,
    ) -> Result<Vec<Option<SequencerAddress>>, Error>;
}
