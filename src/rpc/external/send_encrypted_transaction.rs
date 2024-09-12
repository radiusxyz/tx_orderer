use std::str::FromStr;

use pvde::{
    encryption::{
        poseidon_encryption,
        poseidon_encryption_zkp::{
            verify as verify_poseidon_encryption, PoseidonEncryptionPublicInput,
        },
    },
    num_bigint::BigUint,
    poseidon::hash,
    time_lock_puzzle::{
        key_validation_zkp::verify as verify_key_validation,
        sigma_protocol::{verify as verify_sigma_protocol, SigmaProtocolParam},
        solve_time_lock_puzzle,
    },
};

use crate::{
    rpc::{cluster::SyncEncryptedTransaction, prelude::*},
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendEncryptedTransactionMessage {
    pub rollup_id: String,
    pub encrypted_transaction: EncryptedTransaction,
    pub time_lock_puzzle: TimeLockPuzzle,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendEncryptedTransaction {
    pub message: SendEncryptedTransactionMessage,
    pub signature: Signature,
}

impl SendEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "send_encrypted_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // // get rollup info for address and chain type
        // // verify siganture
        // parameter.signature.verify_signature(
        //     serialize_to_bincode(&parameter.message)?.as_slice(),
        //     parameter.message.address.as_slice(),
        //     parameter.message.chain_type,
        // )?;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.message.rollup_id)?;

        let cluster_metadata = ClusterMetadataModel::get(
            &parameter.message.rollup_id,
            rollup_metadata.block_height(),
        )?;
        match cluster_metadata.is_leader() {
            true => {
                let transaction_order = rollup_metadata.issue_transaction_order();
                let order_hash = rollup_metadata.issue_order_hash(
                    &parameter
                        .message
                        .encrypted_transaction
                        .raw_transaction_hash(),
                );
                let rollup_block_height = rollup_metadata.block_height();
                rollup_metadata.update()?;

                let order_commitment_data = OrderCommitmentData {
                    rollup_id: parameter.message.rollup_id.clone(),
                    block_height: rollup_block_height,
                    transaction_order,
                    previous_order_hash: order_hash,
                };
                let order_commitment = OrderCommitment {
                    data: order_commitment_data,
                    signature: vec![].into(), // Todo: Signature
                };

                EncryptedTransactionModel::put(
                    &parameter.message.rollup_id,
                    rollup_block_height,
                    transaction_order,
                    parameter.message.encrypted_transaction.clone(),
                    Some(parameter.message.time_lock_puzzle.clone()),
                )?;

                let raw_transaction = decrypt_transaction(
                    parameter.message.encrypted_transaction.clone(),
                    parameter.message.time_lock_puzzle.clone(),
                    context.config().is_using_zkp(),
                    &Some(PvdeParams::default()),
                )?;

                RawTransactionModel::put(
                    &parameter.message.rollup_id,
                    rollup_block_height,
                    transaction_order,
                    raw_transaction,
                )?;

                // Sync Transaction
                Self::sync_encrypted_transaction(
                    parameter.clone(),
                    order_commitment.clone(),
                    cluster_metadata,
                );

                Ok(order_commitment)
            }
            false => {
                let leader_rpc_url = cluster_metadata.leader().ok_or(Error::EmptyLeaderRpcUrl)?;
                let client = RpcClient::new(leader_rpc_url)?;
                let response: OrderCommitment = client
                    .request(SendEncryptedTransaction::METHOD_NAME, parameter.clone())
                    .await?;

                Ok(response)
            }
        }
    }

    pub fn sync_encrypted_transaction(
        parameter: Self,
        order_commitment: OrderCommitment,
        cluster_metadata: ClusterMetadata,
    ) {
        tokio::spawn(async move {
            let rpc_parameter = SyncEncryptedTransaction {
                rollup_id: parameter.message.rollup_id,
                encrypted_transaction: parameter.message.encrypted_transaction,
                time_lock_puzzle: parameter.message.time_lock_puzzle,
                order_commitment,
            };

            for follower in cluster_metadata.followers() {
                let rpc_parameter = rpc_parameter.clone();

                tokio::spawn(async move {
                    let client = RpcClient::new(follower.unwrap()).unwrap();
                    let _ = client
                        .request::<SyncEncryptedTransaction, ()>(
                            SyncEncryptedTransaction::METHOD_NAME,
                            rpc_parameter,
                        )
                        .await
                        .unwrap();
                });
            }
        });
    }
}

pub fn decrypt_transaction(
    encrypted_transaction: EncryptedTransaction,
    time_lock_puzzle: TimeLockPuzzle,
    is_using_zkp: bool,
    pvde_params: &Option<PvdeParams>,
) -> Result<RawTransaction, Error> {
    let time_lock_puzzle = time_lock_puzzle.clone();
    let encrypted_data = encrypted_transaction.encrypted_data().clone();
    let open_data = encrypted_transaction.open_data().clone();

    let o = BigUint::from_str(time_lock_puzzle.o()).unwrap();
    let t = time_lock_puzzle.t();
    let n = BigUint::from_str(time_lock_puzzle.n()).unwrap();
    let solved_k = solve_time_lock_puzzle(o, t, n);
    let solved_k_hash_value = hash::hash(solved_k.clone());

    let decrypted_data = poseidon_encryption::decrypt(
        encrypted_data.clone().into_inner().as_str(),
        &solved_k_hash_value,
    );

    // TODO(jaemin): verify zkp(modify pvde library)
    match is_using_zkp {
        true => {
            let pvde_params = pvde_params.as_ref().unwrap();
            let key_validation_zkp_param = pvde_params
                .key_validation_zkp_param()
                .as_ref()
                .unwrap()
                .clone();
            let key_validation_verify_key = pvde_params
                .key_validation_verifying_key()
                .as_ref()
                .unwrap()
                .clone();

            let poseidon_encryption_zkp_param = pvde_params
                .poseidon_encryption_zkp_param()
                .as_ref()
                .unwrap()
                .clone();

            let poseidon_encryption_verify_key = pvde_params
                .poseidon_encryption_verifying_key()
                .as_ref()
                .unwrap()
                .clone();

            let time_lock_puzzle_param = pvde_params
                .time_lock_puzzle_param()
                .as_ref()
                .unwrap()
                .clone();

            let pvde_zkp = encrypted_transaction.pvde_zkp().unwrap();

            let sigma_protocol_public_input =
                pvde_zkp.public_input().to_sigma_protocol_public_input();

            let sigma_protocol_param = SigmaProtocolParam {
                n: time_lock_puzzle_param.n.clone(),
                g: time_lock_puzzle_param.g.clone(),
                y_two: time_lock_puzzle_param.y_two.clone(),
            };
            let is_valid =
                verify_sigma_protocol(&sigma_protocol_public_input, &sigma_protocol_param);

            if !is_valid {
                return Err(Error::PvdeZkpInvalid);
            }
            // log::info!("Done verify_sigma_protocol: {:?}", is_valid);

            let key_validation_public_input =
                pvde_zkp.public_input().to_key_validation_public_input();
            // let key_validation_public_input = KeyValidationPublicInput {
            //     k_two: pvde_zkp.public_input.k_two.clone(),
            //     k_hash_value: pvde_zkp.public_input.k_hash_value.clone(),
            // };
            let is_valid = verify_key_validation(
                &key_validation_zkp_param,
                &key_validation_verify_key,
                &key_validation_public_input,
                &pvde_zkp.time_lock_puzzle_proof().clone().into_inner(),
            );

            if !is_valid {
                return Err(Error::PvdeZkpInvalid);
            }
            // log::info!("Done verify_key_validation: {:?}", is_valid);

            let poseidon_encryption_public_input = PoseidonEncryptionPublicInput {
                encrypted_data: encrypted_data.clone().into_inner(),
                k_hash_value: pvde_zkp.public_input().k_hash_value().clone(),
            };
            let is_valid = verify_poseidon_encryption(
                &poseidon_encryption_zkp_param,
                &poseidon_encryption_verify_key,
                &poseidon_encryption_public_input,
                &pvde_zkp.encryption_proof().clone().into_inner(),
            );

            if !is_valid {
                return Err(Error::PvdeZkpInvalid);
            }
            // log::info!("Done verify_poseidon_encryption: {:?}", is_valid);
        }
        false => {}
    }

    // TODO(jaemin): generalize
    let eth_encrypt_data: EthEncryptData = serde_json::from_str(&decrypted_data).unwrap();
    let ressembled_raw_transaction = match open_data {
        OpenData::Eth(open_data) => open_data.to_raw_transaction(&eth_encrypt_data),
        _ => unreachable!(),
    };
    let eth_raw_transaction = EthRawTransaction::from(to_raw_tx(ressembled_raw_transaction));
    let raw_transaction = RawTransaction::from(eth_raw_transaction);

    Ok(raw_transaction)
}
