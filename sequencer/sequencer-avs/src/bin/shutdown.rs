use std::env;

use json_rpc::RpcClient;
use sequencer_avs::{error::Error, rpc::internal::Deregister};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let internal_rpc_url = arguments
        .get(0)
        .expect("Provide the internal sequencer RPC URL")
        .to_owned();

    let rpc_client = RpcClient::new(internal_rpc_url)?;
    rpc_client
        .request(Deregister::METHOD_NAME, Deregister)
        .await?;

    Ok(())
}
