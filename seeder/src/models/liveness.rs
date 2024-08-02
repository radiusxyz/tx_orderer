use crate::models::prelude::*;

// TODO:
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessInfo {
    pub cluster_type: ClusterType,

    pub provider_rpc_url: IpAddress,
    pub provider_websocket_url: IpAddress,

    pub liveness_contract_address: Option<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessModel {
    pub liveness_info_list: Vec<LivenessInfo>,
}

impl LivenessModel {
    pub fn add(liveness_info: LivenessInfo) -> Result<(), DbError> {
        match Self::get_mut() {
            Ok(mut liveness_model) => {
                liveness_model.liveness_info_list.push(liveness_info);
                liveness_model.update()?;
            }
            Err(_) => {
                let liveness_model = Self {
                    liveness_info_list: vec![liveness_info],
                };
                liveness_model.put()?;
            }
        }

        Ok(())
    }
}

impl LivenessModel {
    pub const ID: &'static str = stringify!(LivenessModel);

    pub fn get() -> Result<Self, DbError> {
        let key = Self::ID;

        match database()?.get(&key) {
            Ok(liveness_model) => Ok(liveness_model),
            Err(_) => Ok(Self {
                liveness_info_list: vec![],
            }),
        }
    }

    pub fn get_mut() -> Result<Lock<'static, Self>, DbError> {
        let key = Self::ID;

        match database()?.get_mut(&key) {
            Ok(liveness_model) => Ok(liveness_model),
            Err(_) => {
                let liveness_model = Self {
                    liveness_info_list: vec![],
                };
                liveness_model.put()?;
                database()?.get_mut(&key)
            }
        }
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = Self::ID;
        database()?.put(&key, self)
    }
}
