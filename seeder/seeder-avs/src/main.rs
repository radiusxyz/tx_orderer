use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use database::Database;
use seeder_avs::{api::*, config::Config, error::Error};
use ssal::ethereum::Endpoint;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    let config = Config::load("Config.toml").map_err(Error::boxed)?;
    let database = Database::new(config.database_path()).map_err(Error::boxed)?;
    let listener = TcpListener::bind(config.seeder_address())
        .await
        .map_err(Error::boxed)?;
    let app = Router::new()
        .route(Endpoint::REGISTER, post(Register::handler))
        .route(Endpoint::DEREGISTER, post(Deregister::handler))
        .route(Endpoint::ADDRESS_LIST, get(AddressList::handler))
        .with_state(database);

    tracing::info!("Starting the seeder server at {}", config.seeder_address());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(Error::boxed)?;
    Ok(())
}
