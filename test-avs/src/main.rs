use std::{io::stdin, str::FromStr};

use chrono::Utc;
use eigen_client_elcontracts::{
    reader::ELChainReader,
    writer::{ELChainWriter, Operator},
};
use eigen_utils::binding::ECDSAStakeRegistry::{self, SignatureWithSaltAndExpiry};
use rand::RngCore;
use ssal::avs::{types::*, SsalClient, SsalEventListener};

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let ssal_client = SsalClient::new(
        "http://127.0.0.1:8545",
        "/home/kanet/Projects/sequencer-framework/sequencer/sequencer-avs/keys/sequencer_1",
        "sequencer_1",
        "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
        "0x95401dc811bb5740090279Ba06cfA8fcF6113778",
        "http://127.0.0.1:3000",
    )?;

    // Register Operator
    let default_slasher = Address::ZERO; // We don't need slasher for our example.
    let default_strategy = Address::ZERO; // We don't need strategy for our example.
    let delegation_manager_contract_address =
        Address::from_str("0x9E545E3C0baAB3E08CdfD552C960A1050f373042")?;
    let avs_directory_contract_address =
        Address::from_str("0x95401dc811bb5740090279Ba06cfA8fcF6113778")?;

    let elcontracts_reader_instance = ELChainReader::new(
        default_slasher,
        delegation_manager_contract_address,
        avs_directory_contract_address,
        "http::/127.0.0.1:8545".parse()?,
    );

    let elcontracts_writer_instance = ELChainWriter::new(
        delegation_manager_contract_address,
        default_strategy,
        elcontracts_reader_instance.clone(),
        "http://127.0.0.1:8545".parse()?,
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_owned(),
    );

    let operator = Operator::new(
        ssal_client.address(),
        ssal_client.address(),
        Address::ZERO,
        0u32,
        None,
    );

    let _tx_hash = elcontracts_writer_instance
        .register_as_operator(operator)
        .await;

    let mut salt = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let salt = FixedBytes::from_slice(&salt);
    let now = Utc::now().timestamp();
    let expiry: U256 = U256::from(now + 3600);
    let digest_hash = elcontracts_reader_instance
        .calculate_operator_avs_registration_digest_hash(
            ssal_client.address(),
            Address::from_str("0x95401dc811bb5740090279Ba06cfA8fcF6113778")?,
            salt,
            expiry,
        )
        .await
        .expect("not able to calculate operator ");

    let signature = ssal_client.signer().sign_hash(&digest_hash).await?;

    let operator_signature = SignatureWithSaltAndExpiry {
        signature: signature.as_bytes().into(),
        salt,
        expiry: expiry,
    };

    let contract_ecdsa_stake_registry = ECDSAStakeRegistry::new(
        Address::from_str("0xa82fF9aFd8f496c3d6ac40E2a0F282E47488CFc9")?,
        ssal_client.provider(),
    );
    println!("initialize new ecdsa ");

    // If you wish to run on holesky, please deploy the stake registry contract(it's not deployed right now)
    // and uncomment the gas and gas_price
    let registeroperator_details = contract_ecdsa_stake_registry
        .registerOperatorWithSignature(ssal_client.address(), operator_signature);
    let _tx = registeroperator_details
        // .gas(300000)
        // .gas_price(20000000000)
        .send()
        .await?
        .get_receipt()
        .await?;

    tracing::info!("Operator registered succesfully");

    let event_listener = SsalEventListener::connect(
        "ws://127.0.0.1:8545",
        "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
        "0x95401dc811bb5740090279Ba06cfA8fcF6113778",
    )
    .await?;

    tokio::spawn({
        let ssal_client = ssal_client.clone();

        async move {
            event_listener.init(callback, ssal_client).await.unwrap();
        }
    });

    loop {
        let command = input("1. Initialize a new cluster\n2. Register a sequencer\n3. Deregister a sequencer\n4. Register a block commitment")?;

        match command.trim() {
            "1" => initialize_cluster(&ssal_client).await?,
            "2" => register_sequencer(&ssal_client).await?,
            "3" => deregister_sequencer(&ssal_client).await?,
            "4" => register_block_commitment(&ssal_client).await?,
            _ => continue,
        }
    }
}

fn input(command: &'static str) -> Result<String, Error> {
    println!("{}", command);

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(input.trim().to_owned())
}

async fn initialize_cluster(client: &SsalClient) -> Result<(), Error> {
    let sequencer_rpc_url = input("sequencer_rpc_url")?;
    let sequencer_address = input("sequencer_address")?;
    let rollup_address = input("rollup_address")?;

    client
        .initialize_cluster(sequencer_rpc_url, sequencer_address, rollup_address)
        .await?;

    Ok(())
}

async fn register_sequencer(client: &SsalClient) -> Result<(), Error> {
    let cluster_id = input("cluster_id")?;
    let sequencer_address = input("sequencer_address")?;

    client
        .register_sequencer(cluster_id, sequencer_address)
        .await?;

    Ok(())
}

async fn deregister_sequencer(client: &SsalClient) -> Result<(), Error> {
    let cluster_id = input("cluster_id")?;
    let sequencer_address = input("sequencer_address")?;

    client
        .deregister_sequencer(cluster_id, sequencer_address)
        .await?;

    Ok(())
}

async fn register_block_commitment(client: &SsalClient) -> Result<(), Error> {
    let block_commitment = input("block_commitment")?;
    let block_number = input("block_number")?.parse::<u64>()?;
    let rollup_id = input("rollup_id")?.parse::<u32>()?;
    let cluster_id = input("cluster_id")?;

    client
        .register_block_commitment(block_commitment, block_number, rollup_id, cluster_id)
        .await?;

    Ok(())
}

async fn callback(event: SsalEventType, context: SsalClient) {
    match event {
        SsalEventType::NewBlock(block) => {
            let _block_number = block.header.number.unwrap();
            let _sequencer_list = context
                .get_sequencer_list(
                    "0x38a941d2d4959baae54ba9c14502abe54ffd4ad0db290295f453ef9d7d5a3f2d",
                )
                .await
                .unwrap();

            // tracing::info!(
            //     "Block Number: {}\nSequencer List: {:?}",
            //     block_number,
            //     sequencer_list
            // );
        }
        SsalEventType::InitializeCluster((event, _log)) => {
            tracing::info!("{}", event.clusterID.to_string());
        }
        SsalEventType::BlockCommitment((event, _log)) => {
            match context
                .respond_to_task(
                    event.task,
                    event.taskIndex,
                    "0x38a941d2d4959baae54ba9c14502abe54ffd4ad0db290295f453ef9d7d5a3f2d",
                )
                .await
            {
                Ok(_) => {
                    tracing::info!("Successfully registered the block commitment");
                }
                Err(error) => tracing::error!("{}", error),
            }
        }
    }
}
