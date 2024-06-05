use crate::rpc::prelude::*;

/// After the first updating cluster metadata, the sequencer will no longer return
/// `SequencerStatus::Uninitialized` to both users and rollups.
pub fn update_cluster_metadata(
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
) -> Result<(), RpcError> {
    // Create a new block metadata.
    let block_metadata = BlockMetadata::default();
    block_metadata.put(rollup_block_number)?;

    // Get the sequencer list corresponding to the SSAL block number.
    let sequencer_list = SequencerList::get(ssal_block_number)?;

    // Calculate the leader index using the remainder operation.
    let leader_index = rollup_block_number % sequencer_list.len();
    let (leader, followers) = sequencer_list.split_leader_from_followers(leader_index);

    // Check if the current sequencer is the leader.
    let me = Me::get()?;
    let is_leader = me.into_public_key() == leader.0;

    // Create a new cluster metadata.
    ClusterMetadata::new(
        ssal_block_number,
        rollup_block_number,
        leader,
        followers,
        is_leader,
    )
    .put()?;
    Ok(())
}
