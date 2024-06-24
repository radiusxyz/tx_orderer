use std::io::stdin;

use ssal::avs::{types::SsalEventType, SsalClient, SsalEventListener};

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let ssal_client = SsalClient::new(
        "http://127.0.0.1:8545",
        // "/home/kanet/Projects/sequencer-framework/sequencer/sequencer-avs/keys/sequencer_1",
        // "sequencer_1",
        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
        "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
        "0x9E545E3C0baAB3E08CdfD552C960A1050f373042",
        "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
        "0x95401dc811bb5740090279Ba06cfA8fcF6113778",
        "http://127.0.0.1:3000",
    )?;

    ssal_client.register_as_operator().await?;

    let ssal_event_listener = SsalEventListener::connect(
        "ws://127.0.0.1:8545",
        "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
        "0x95401dc811bb5740090279Ba06cfA8fcF6113778",
    )
    .await?;

    tokio::spawn({
        let ssal_client = ssal_client.clone();

        async move {
            ssal_event_listener
                .init(callback, ssal_client)
                .await
                .unwrap();
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
            // let _sequencer_list = context
            //     .get_sequencer_list(
            //         "0x38a941d2d4959baae54ba9c14502abe54ffd4ad0db290295f453ef9d7d5a3f2d",
            //     )
            //     .await
            //     .unwrap();

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
