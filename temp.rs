
async fn check_failover(app_state: &AppState) -> Result<(), Error> {
    // Check if the sequencer has failed previously.
    match ClusterMetadata::get_mut().ok() {
        Some(mut cluster_metadata) => {
            tracing::warn!("Found a saved cluster metadata. Recovering the previous state..");

            let ssal_block_height = cluster_metadata.ssal_block_height;
            let rollup_block_height = cluster_metadata.rollup_block_height;

            let cluster = cluster_metadata
                .update(
                    app_state.ssal_client().address(),
                    app_state.config().cluster_id(),
                    ssal_block_height,
                    rollup_block_height,
                )
                .await?;
            app_state.update_cluster(cluster).await;

            // TODO:
            // Check if the `build_block` request had been sent by the rollup before the leader recovered.

            tracing::info!("Succesfully recovered the previous state.");
        }
        None => tracing::info!("Initializing the sequencer.."),
    }

    Ok(())
}

async fn register_as_operator(app_state: &AppState) -> Result<(), Error> {
    match app_state.ssal_client().is_operator().await? {
        true => {
            tracing::info!(
                "Already registered as an operator. Skipping the operator registration.."
            );
        }
        false => {
            app_state.ssal_client().register_as_operator().await?;
            tracing::info!("Successfully registered as an operator.");
        }
    }

    Ok(())
}

async fn register_on_avs(app_state: &AppState) -> Result<(), Error> {
    match app_state.ssal_client().is_avs().await? {
        true => {
            tracing::info!("Already registered on AVS. Skipping the AVS registration.");
        }
        false => {
            app_state.ssal_client().register_on_avs().await?;
            tracing::info!("Successfully registered on AVS.");
        }
    }

    Ok(())
}

async fn register_sequencer(app_state: &AppState) -> Result<(), Error> {
    match app_state
        .ssal_client()
        .is_registered(
            app_state.config().cluster_id(),
            app_state.ssal_client().address(),
        )
        .await?
    {
        true => {
            tracing::info!("Already registered on the SSAL contract. Skipping the registration..")
        }
        false => {
            app_state
                .ssal_client()
                .register_sequencer(
                    app_state.config().cluster_id(),
                    app_state.config().sequencer_rpc_url(),
                )
                .await?;
            tracing::info!("Successfully registered the sequencer on SSAL contract.");
        }
    }

    Ok(())
}

