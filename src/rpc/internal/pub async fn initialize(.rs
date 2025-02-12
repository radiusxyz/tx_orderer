pub async fn initialize(
    context: AppState,
    platform: Platform,
    service_provider: ServiceProvider,
    liveness_info: LivenessRadius,
) -> Result<(), Error> {
    let signing_key = &context.config().signing_key;
    let signer =
        PrivateKeySigner::from_str(platform.into(), signing_key).expect("Invalid signing key");

    context
        .add_signer(platform, signer)
        .await
        .expect("Failed to add signer");

    let liveness_service_manager_client = Arc::new(
        Self::new(
            platform,
            service_provider,
            liveness_info.clone(),
            signing_key,
            context.seeder_client().clone(),
        )
        .expect("Failed to create liveness client"),
    );

    let current_block_height = liveness_service_manager_client
        .publisher()
        .get_block_number()
        .await
        .expect("Failed to get block number");

    let block_margin: u64 = liveness_service_manager_client
        .publisher()
        .get_block_margin()
        .await
        .expect("Failed to get block margin")
        .try_into()
        .expect("Failed to convert block margin");

    let cluster_id_list = ClusterIdList::get_or(
        liveness_service_manager_client.platform(),
        liveness_service_manager_client.service_provider(),
        ClusterIdList::default,
    )
    .expect("Failed to get cluster id list");

    for cluster_id in cluster_id_list.iter() {
        if let Err(e) = initialize_new_cluster(
            context.clone(),
            &liveness_service_manager_client,
            cluster_id,
            current_block_height,
            block_margin,
        )
        .await
        {
            tracing::error!(
                "Failed to initialize new cluster for cluster_id: {:?} - {:?}",
                cluster_id,
                e
            );
        }
    }

    context
        .add_liveness_service_manager_client(platform, service_provider, liveness_service_manager_client.clone())
        .await
        .expect("Failed to add liveness client");

    let event_listener_context = context.clone();
    let event_listener_client = liveness_service_manager_client.clone();

    let handle = tokio::spawn(async move {
        tracing::info!(
            "Initializing the liveness event listener for {:?}, {:?}..",
            platform,
            service_provider
        );

        loop {
            let result = event_listener_client
                .subscriber()
                .initialize_event_handler(
                    callback,
                    (
                        event_listener_context.clone(),
                        event_listener_client.clone(),
                    ),
                )
                .await;

            if let Err(error) = result {
                tracing::error!(
                    "Liveness event listener encountered an error for {:?}, {:?} - {:?}",
                    platform,
                    service_provider,
                    error
                );
            }

            tracing::warn!(
                "Reconnecting the liveness event listener for {:?}, {:?}..",
                platform,
                service_provider
            );

            sleep(Duration::from_secs(5)).await;
        }
    });

    tokio::spawn(async move {
        if let Err(e) = handle.await {
            tracing::warn!(
                "Event listener crashed, attempting to restart for {:?}, {:?}.. - {:?}",
                platform,
                service_provider,
                e
            );
            sleep(Duration::from_secs(5)).await;
            let _ = Self::initialize(context, platform, service_provider, liveness_info).await;
        }
    });

    Ok(())
}
