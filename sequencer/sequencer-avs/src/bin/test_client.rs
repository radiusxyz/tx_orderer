use std::env;

use database::Database;
use json_rpc::RpcClient;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use sequencer_avs::{
    config::Config, error::Error, rpc::external::SendTransaction, state::AppState, task::TraceExt,
    types::*,
};
use ssal::avs::{types::SsalEventType, SsalClient, SsalEventListener};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    std::panic::set_hook(Box::new(|panic_info| tracing::error!("{}", panic_info)));

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the config file path.")
        .to_owned();

    // Load the configuration from the path.
    let config = Config::load(&config_path)?;
    tracing::info!(
        "Successfully loaded the configuration file at {}",
        config_path,
    );

    // Initialize the database.
    Database::new(config.database_path())?.init();
    tracing::info!(
        "Succesfully initialized the database at {:?}",
        config.database_path(),
    );

    // Initialize the SSAL client.
    let ssal_client = SsalClient::new(
        config.ethereum_rpc_url(),
        config.signing_key(),
        config.seeder_rpc_url(),
        config.ssal_contract_address(),
        config.delegation_manager_contract_address(),
        config.stake_registry_contract_address(),
        config.avs_directory_contract_address(),
        config.avs_contract_address(),
    )?;
    tracing::info!("Successfully initialized the SSAL client");

    // Initialize an application-wide state instance.
    let app_state = AppState::new(config, ssal_client, None);

    // Initialize the event manager.
    event_manager(app_state.clone());

    // Initialize the random seed.
    let mut seed = rand::thread_rng();

    // Start sending the transaction.
    loop {
        let current_ssal_block_number = SsalBlockNumber::get().ok_or_trace();

        if let Some(ssal_block_number) = current_ssal_block_number {
            let sequencer_list = SequencerList::get(ssal_block_number - BLOCK_MARGIN).ok();

            if let Some(sequencer_list) = sequencer_list {
                send_transaction(sequencer_list, &mut seed).await.ok();
            }
        }

        sleep(Duration::from_millis(600)).await;
    }
}

async fn send_transaction(
    sequencer_list: SequencerList,
    seed: &mut ThreadRng,
) -> Result<(), Error> {
    let sequencer_list = sequencer_list.into_inner();
    let sequencer = sequencer_list.choose(seed);
    if let Some((_address, rpc_url)) = sequencer {
        if let Some(rpc_url) = rpc_url {
            let transaction = UserTransaction::new(
                    EncryptedTransaction::new("04ef55bd572cf13ee6e71e6fc99f4c81f651956fade173ae258d2146b451456565b198527a90fbe9ed82e79b287b8825b49590cc79f6357c0bdd04b8797829712adbecf768d946a083e17775bde91fc0cd16d863dd82598de10e31bfab47102a561d8c5950193c39177691a3c80e85a2c414c1f0f8115afd8b60855ccb1be61c2a3a5691cde8757cd8ede1741dd1e803184695552e7efe78d1b6c03b811f772f24ffc9c2eebf2a66b4b49f4c47c86aa3fbb6631de0dd05c70f058a17fd92df56901eb15b753b2c9863a200692d0b03322ae1fb8a5cdb047c23ae048d65e8c972f70a979e6bb514445a11a86d05f0db6933f7331ec70c0f4c6f1378473722925821395a320f7d110d3714ed82bd586791995c6583b8637b6d54ac64ada0409c503e9f8981f78a84121af279a494394094e4a9682fa6b30237b76f3f4a05687b374e9cee2ff806f55d97a288546e58f11370300d10605e9706659ee65c0d5a824c1fe732c14b345cff07a5226d75781f1596210b878543e9fdfdc0518cb11cb460ada14165453571d14fa4a60bc79e93270f3e397322bdeb8a731723a00dfab35f3e0c6b66d2483628a56a966843bbd653cb1bd1ac9154ee2b6d290fa6ac49525caaadd9ba352437a76be70aee825c4358cdc9b2eadf32c203b486b2cb5393d73f504a02a47019072e96b4a2ae8a906865def86d214b3a305ba44d6522150c5868"),
                    TimeLockPuzzle::new(
                        22,
                        "12457311715257449457748751984254622508267155894507658142695068003493042195405329207711366022835762377245277882484119178471430417559474738331984329761022187066519485013748894706305829380840193820890825899928855510737760426323558539179908225333013645902208278947606275823761478198422445543050872542413949439903608920146934349284533080536914298966417758077170245244490396562694211535110579413584287824647888452220286793100316938024359130504308887346003994598050645271382321919886352017301366008280956763147859805767018208718575759826940798951801158140056465786453627045574818695000225389991680219160348455850457454655",
                        "2025248438650826187916433341304836251499903908136140620190225679742998627952261293517599834537847933074023268746097615547477918545771661724277068909300914207042202270430377935887117773540977453830744227374953807472502106790159585523442864538764741325253270129883307270347019748164333643663580966609532162984413354607666047216242926253671347602354979303095459943544043702401177588210085535811793517573250134654357949162914951748375241718970707463457449447959617669536797071325145499926957188895602050452436800605688032168511552979105389392764220149523107905574258809617228115042581847973213995828930403395363121196763",
                    ),
                    Nonce::new("e90167be0f9daa0be9c9ca4b1716f9541442eccc4b7fbe9224a2b6212f229b16"),
                );

            let rpc_client = RpcClient::new(rpc_url)?;
            let rpc_method = SendTransaction { transaction };
            let order_commitment: OrderCommitment = rpc_client
                .request(SendTransaction::METHOD_NAME, rpc_method)
                .await?;

            tracing::info!("{:?}", order_commitment);
        }
    }

    Ok(())
}

fn event_manager(context: AppState) {
    tokio::spawn(async move {
        loop {
            let ssal_event_listener = SsalEventListener::connect(
                context.config().ethereum_websocket_url(),
                context.config().ssal_contract_address(),
                context.config().avs_contract_address(),
            )
            .await
            .ok_or_trace();

            if let Some(ssal_event_listener) = ssal_event_listener {
                ssal_event_listener
                    .init(event_callback, context.clone())
                    .await
                    .ok_or_trace();
            }

            sleep(Duration::from_secs(3)).await;
            tracing::warn!("Reconnecting the event listener..");
        }
    });
}

async fn event_callback(event_type: SsalEventType, context: AppState) {
    match event_type {
        SsalEventType::NewBlock(block) => {
            if let Some(block_number) = block.header.number {
                let sequencer_list = context
                    .ssal_client()
                    .get_sequencer_list(context.config().cluster_id())
                    .await
                    .ok_or_trace();

                if let Some(sequencer_list) = sequencer_list {
                    SequencerList::from(sequencer_list)
                        .put(block_number)
                        .ok_or_trace();
                }

                SsalBlockNumber::from(block_number).put().unwrap();
            }
        }
        _ => {}
    }
}
