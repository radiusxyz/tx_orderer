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
    models::{
        EncryptedTransactionModel, RawTransactionModel, RollupMetadataModel, TransactionModel,
    },
    rpc::prelude::*,
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendEncryptedTransaction {
    rollup_id: RollupId,
    encrypted_transaction: EncryptedTransaction,
    time_lock_puzzle: TimeLockPuzzle,
}

impl SendEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "send_encrypted_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<OrderCommitment, RpcError> {
        let parameter = parameter.parse::<SendEncryptedTransaction>()?;

        // TODO: 1. verify encrypted_transaction

        // 2. Issue order_commitment
        let block_height = context.block_height(&parameter.rollup_id).await?;

        let raw_transaction_hash = parameter.encrypted_transaction.raw_transaction_hash();

        let mut rollup_metadata_model = RollupMetadataModel::get_mut(&parameter.rollup_id)?;
        let transaction_order = rollup_metadata_model.rollup_metadata().transaction_order();

        let previous_order_hash = rollup_metadata_model.rollup_metadata().order_hash();
        let issued_order_hash = previous_order_hash.issue_order_hash(&raw_transaction_hash);

        let mut new_rollup_metadata_model = rollup_metadata_model.rollup_metadata().clone();
        new_rollup_metadata_model.update_order_hash(issued_order_hash.clone());
        new_rollup_metadata_model.increase_transaction_order();

        rollup_metadata_model.update_rollup_metadata(new_rollup_metadata_model);
        rollup_metadata_model.update()?;

        let order_commitment_data = OrderCommitmentData {
            rollup_id: parameter.rollup_id.clone(),
            block_height,
            transaction_order: transaction_order.clone(),
            previous_order_hash: issued_order_hash.clone(),
        };
        println!("issued_order_hash: {:?}", issued_order_hash);
        let order_commitment_signature = Signature::default(); // TODO
        let order_commitment = OrderCommitment {
            data: order_commitment_data,
            signature: order_commitment_signature,
        };

        // 3. Save encrypted_transaction
        let encrypted_transaction_model = EncryptedTransactionModel::new(
            parameter.encrypted_transaction.clone(),
            parameter.time_lock_puzzle.clone(),
        );
        encrypted_transaction_model.put(&parameter.rollup_id, &block_height, &transaction_order)?;

        // 4. Save raw_transaction
        let raw_transaction = decrypt_transaction(
            parameter.encrypted_transaction.clone(),
            parameter.time_lock_puzzle.clone(),
            context.config().is_using_zkp(),
            context.pvde_params().load().as_ref(),
        )?;
        let raw_transaction_model = RawTransactionModel::new(raw_transaction);
        raw_transaction_model.put(&parameter.rollup_id, &block_height, &transaction_order)?;

        // 4. Sync transaction
        let cluster_id = context.get_cluster_id(&parameter.rollup_id).await?;
        let cluster = context.get_cluster(&cluster_id).await?;

        syncer::sync_transaction(
            cluster.clone(),
            parameter.rollup_id.clone(),
            TransactionModel::Encrypted(encrypted_transaction_model),
            order_commitment.clone(),
        );

        syncer::sync_transaction(
            cluster.clone(),
            parameter.rollup_id.clone(),
            TransactionModel::Raw(raw_transaction_model),
            order_commitment.clone(),
        );

        Ok(order_commitment)
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
