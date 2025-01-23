if sequencer_address == block_creator_address {
            let paltform = rollup.validation_info.platform;
            let validation_service_provider = rollup.validation_info.validation_service_provider;

            let validation_info =
                ValidationInfo::get(paltform, validation_service_provider).unwrap();

            if rollup_block_height % 100 == 0 {
                match validation_info {
                    // TODO: we have to manage the nonce for the register block commitment.
                    ValidationInfo::EigenLayer(_) => {
                        let validation_client: validation::eigenlayer::ValidationClient = context
                            .get_validation_client(paltform, validation_service_provider)
                            .await
                            .unwrap();

                        validation_client
                            .publisher()
                            .register_block_commitment(
                                rollup.cluster_id,
                                rollup.rollup_id,
                                rollup_block_height,
                                block_commitment,
                            )
                            .await
                            .unwrap();
                    }
                    ValidationInfo::Symbiotic(_) => {
                        let validation_client: validation::symbiotic::ValidationClient = context
                            .get_validation_client(paltform, validation_service_provider)
                            .await
                            .unwrap();

                        for _ in 0..10 {
                            match validation_client
                                .publisher()
                                .register_block_commitment(
                                    &rollup.cluster_id,
                                    &rollup.rollup_id,
                                    rollup_block_height,
                                    block_commitment,
                                )
                                .await
                                .map_err(|error| error.to_string())
                            {
                                Ok(transaction_hash) => {
                                    tracing::info!(
                                        "Registered block commitment - transaction hash: {:?}",
                                        transaction_hash
                                    );
                                    break;
                                }
                                Err(error) => {
                                    tracing::warn!("{:?}", error);
                                    sleep(Duration::from_secs(2)).await;
                                }
                            }
                        }
                    }
                }
            }
        }