use ssal::ethereum::{types::*, SsalClient};

use crate::{error::Error, types::*};

pub fn init(ssal_client: &SsalClient) {
    let ssal_client = ssal_client.clone();
    tokio::spawn(async move {
        ssal_client
            .sequencer_list_subscriber(handler)
            .await
            .unwrap();
    });
}

async fn handler(
    ssal_block_number: u64,
    sequencer_list: (Vec<PublicKey>, Vec<Option<RpcAddress>>),
) -> Result<(), Error> {
    let sequencer_list = SequencerList::from(sequencer_list);
    sequencer_list.put(ssal_block_number.into())?;
    Ok(())
}
