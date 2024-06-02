use std::env;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let config = Config::load("Config.toml")?;
    let database = Database::new(config.database_path())?;
    let listener = TcpListener::bind(config.seeder_address()).await?;
    let app = Router::new()
        .route("/register", post(Register::handler))
        .route("/deregister", post(Deregister::handler))
        .route("/get-sequencer-list", get(SequencerList::handler))
        .with_state(database);

    tracing::info!("Starting the seeder server at {}", config.seeder_address());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
