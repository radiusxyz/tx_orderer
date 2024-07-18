use json_rpc::RpcClient;
use sequencer::{error::Error, rpc::internal::Deregister};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let rpc_client = RpcClient::new("http://127.0.0.1:7234")?;
    rpc_client
        .request(Deregister::METHOD_NAME, Deregister)
        .await?;

    Ok(())
}
