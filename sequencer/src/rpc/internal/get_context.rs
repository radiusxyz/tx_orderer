use std::collections::HashMap;

use serde_json::{json, Value};

use super::get_cluster_id_list;
use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetContext {}

impl GetContext {
    pub const METHOD_NAME: &'static str = "get_context";

    pub async fn handler(
        _parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<Value, RpcError> {
        let config = context.config();
        let rollup_metadatas = context.rollup_metadatas().await;
        let rollup_cluster_ids = context.rollup_cluster_ids().await;

        let sequencing_infos = context.sequencing_infos().await;
        let sequencing_infos = sequencing_infos
            .clone()
            .into_iter()
            .map(|(sequencing_info_key, sequencing_info)| {
                (sequencing_info_key.to_string(), sequencing_info)
            })
            .collect::<HashMap<String, SequencingInfo>>();

        let get_cluster_id_list = context
            .clusters()
            .await
            .keys()
            .cloned()
            .collect::<Vec<ClusterId>>();

        // let rollup_metadatas = format!("{:?}", rollup_metadatas);

        let result = json!({
          "config": config,
          "rollup_metadatas": rollup_metadatas,
          "rollup_cluster_ids": rollup_cluster_ids,
          "sequencing_infos": sequencing_infos,
          "get_cluster_id_list": get_cluster_id_list,
        });

        Ok(result)
    }
}
