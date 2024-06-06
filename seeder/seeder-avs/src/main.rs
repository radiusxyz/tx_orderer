use database::Database;
use json_rpc::RpcServer;
use seeder_avs::{config::Config, error::Error, rpc::*};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    let config = Config::load("Config.toml").map_err(Error::boxed)?;

    Database::new(config.database_path())?.init();

    RpcServer::new()
        .register_rpc_method::<Register>()?
        .register_rpc_method::<Deregister>()?
        .register_rpc_method::<GetAddressList>()?
        .init(config.seeder_rpc_address())
        .await?;
    Ok(())
}
