use std::{collections::HashMap, fmt::Display};

use crate::models::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequencingInfoModel {
    sequencing_infos: HashMap<SequencingInfoKey, SequencingInfo>,
}

impl SequencingInfoModel {
    pub fn new(sequencing_infos: HashMap<SequencingInfoKey, SequencingInfo>) -> Self {
        Self { sequencing_infos }
    }

    pub fn sequencing_infos(&self) -> &HashMap<SequencingInfoKey, SequencingInfo> {
        &self.sequencing_infos
    }

    pub fn sequencing_infos_mut(&mut self) -> &mut HashMap<SequencingInfoKey, SequencingInfo> {
        &mut self.sequencing_infos
    }
}

impl SequencingInfoModel {
    pub fn add(sequencing_info: SequencingInfo) -> Result<(), DbError> {
        match Self::get_mut() {
            Ok(mut liveness_model) => {
                liveness_model.sequencing_infos_mut().insert(
                    SequencingInfoKey::new(
                        sequencing_info.platform.clone(),
                        sequencing_info.sequencing_function_type.clone(),
                        sequencing_info.service_type.clone(),
                    ),
                    sequencing_info,
                );

                liveness_model.update()?;
            }
            Err(_) => {
                let mut liveness_model = Self::new(HashMap::new());
                liveness_model.sequencing_infos_mut().insert(
                    SequencingInfoKey::new(
                        sequencing_info.platform.clone(),
                        sequencing_info.sequencing_function_type.clone(),
                        sequencing_info.service_type.clone(),
                    ),
                    sequencing_info,
                );
                liveness_model.put()?;
            }
        }

        Ok(())
    }
}

impl SequencingInfoModel {
    pub const ID: &'static str = stringify!(LivenessModel);

    pub fn get() -> Result<Self, DbError> {
        let key = Self::ID;

        match database()?.get(&key) {
            Ok(liveness_model) => Ok(liveness_model),
            Err(_) => Ok(Self {
                sequencing_infos: HashMap::new(),
            }),
        }
    }

    pub fn get_mut() -> Result<Lock<'static, Self>, DbError> {
        let key = Self::ID;

        match database()?.get_mut(&key) {
            Ok(liveness_model) => Ok(liveness_model),
            Err(_) => {
                let liveness_model = Self {
                    sequencing_infos: HashMap::new(),
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
