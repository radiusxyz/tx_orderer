use ssal::avs::types::Address;
use tokio::time::{sleep, Duration};

use crate::types::SequencerList;

const MARGIN: u64 = 3;

pub fn init(my_address: Address, block_number_at_request: u64) {
    tokio::spawn(async move {
        loop {
            let sequencer_list = SequencerList::get(block_number_at_request - MARGIN).ok();
            if let Some(sequencer_list) = sequencer_list {
                match sequencer_list
                    .into_inner()
                    .into_iter()
                    .find(|(address, _rpc_url)| *address == my_address)
                {
                    Some(_) => continue,
                    None => break,
                }
            }

            sleep(Duration::from_secs(5)).await;
        }

        std::process::exit(0);
    });
}
