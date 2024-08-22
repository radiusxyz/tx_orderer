use std::collections::HashMap;

use crate::models::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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
            Ok(mut sequencing_info_model) => {
                sequencing_info_model.sequencing_infos_mut().insert(
                    SequencingInfoKey::new(
                        sequencing_info.platform.clone(),
                        sequencing_info.sequencing_function_type.clone(),
                        sequencing_info.service_type.clone(),
                    ),
                    sequencing_info,
                );

                sequencing_info_model.update()?;
            }
            Err(_) => {
                let mut sequencing_info_model = Self::new(HashMap::new());
                sequencing_info_model.sequencing_infos_mut().insert(
                    SequencingInfoKey::new(
                        sequencing_info.platform.clone(),
                        sequencing_info.sequencing_function_type.clone(),
                        sequencing_info.service_type.clone(),
                    ),
                    sequencing_info,
                );
                sequencing_info_model.put()?;
            }
        }

        Ok(())
    }
}

impl SequencingInfoModel {
    pub const ID: &'static str = stringify!(SequencingInfoModel);

    pub fn get() -> Result<Self, DbError> {
        let key = Self::ID;

        database()?.get(&key)
    }

    pub fn get_mut() -> Result<Lock<'static, Self>, DbError> {
        let key = Self::ID;

        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = Self::ID;
        database()?.put(&key, self)
    }
}
