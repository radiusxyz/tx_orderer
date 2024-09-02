use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Ethereum,
    Local,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ServiceProvider {
    Radius,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SequencingInfoPayload {
    Ethereum(LivenessEthereum),
    Local(LivenessLocal),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessEthereum {
    rpc_url: String,
    websocket_url: String,
    contract_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessLocal {
    rpc_url: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SequencingInfo {
    platform: Platform,
    service_provider: ServiceProvider,
    payload: SequencingInfoPayload,
}

impl<'de> serde::de::Deserialize<'de> for SequencingInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Platform,
            ServiceProvider,
            Payload,
        }

        struct SequencingInfoVisitor;

        impl<'de> serde::de::Visitor<'de> for SequencingInfoVisitor {
            type Value = SequencingInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct SequencingInfo")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut platform = None;
                let mut service_provider = None;
                let mut payload: Option<serde_json::Value> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Platform => {
                            if platform.is_some() {
                                return Err(serde::de::Error::duplicate_field("platform"));
                            }
                            platform = Some(map.next_value()?);
                        }
                        Field::ServiceProvider => {
                            if service_provider.is_some() {
                                return Err(serde::de::Error::duplicate_field("service_provider"));
                            }
                            service_provider = Some(map.next_value()?);
                        }
                        Field::Payload => {
                            if payload.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload = Some(map.next_value()?);
                        }
                    }
                }

                let platform = platform.ok_or(serde::de::Error::missing_field("platform"))?;
                let service_provider =
                    service_provider.ok_or(serde::de::Error::missing_field("service_provider"))?;
                let payload = payload.ok_or(serde::de::Error::missing_field("payload"))?;

                match platform {
                    Platform::Ethereum => {
                        let payload: LivenessEthereum =
                            serde_json::from_value(payload).map_err(serde::de::Error::custom)?;

                        Ok(SequencingInfo {
                            platform,
                            service_provider,
                            payload: SequencingInfoPayload::Ethereum(payload),
                        })
                    }

                    Platform::Local => {
                        let payload: LivenessLocal =
                            serde_json::from_value(payload).map_err(serde::de::Error::custom)?;

                        Ok(SequencingInfo {
                            platform,
                            service_provider,
                            payload: SequencingInfoPayload::Local(payload),
                        })
                    }
                }
            }
        }

        const FIELDS: &[&str] = &["platform", "service_provider", "payload"];

        deserializer.deserialize_struct("SequencingInfo", FIELDS, SequencingInfoVisitor)
    }
}
