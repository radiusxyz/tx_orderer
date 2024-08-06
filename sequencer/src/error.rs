use radius_sequencer_sdk::{json_rpc::Error as RpcError, kvstore::KvStoreError as DbError};

#[derive(Debug)]
pub enum Error {
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Database(DbError),
    RpcError(RpcError),
    Ssal(ssal::avs::Error),
    Uninitialized,
    EmptySequencerList,
    LeaderIndexOutOfBound,
    EmptyLeaderRpcUrl,
    FetchResponse,
    ClusterDown,
    InvalidSequencerPort,

    LoadConfigOption,
    ParseTomlString,

    RemoveConfigDirectory,
    CreateConfigDirectory,
    CreateConfigFile,
    CreatePrivateKeyFile,

    RegisterRpcUrl,
    GetSequencerRpcUrlList,

    GetSequencingInfo,

    LivenessPublisher(radius_sequencer_sdk::liveness::publisher::PublisherError),
}

unsafe impl Send for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<radius_sequencer_sdk::liveness::publisher::PublisherError> for Error {
    fn from(value: radius_sequencer_sdk::liveness::publisher::PublisherError) -> Self {
        Self::LivenessPublisher(value)
    }
}

impl From<DbError> for Error {
    fn from(value: DbError) -> Self {
        Self::Database(value)
    }
}

impl From<RpcError> for Error {
    fn from(value: RpcError) -> Self {
        Self::RpcError(value)
    }
}

impl From<ssal::avs::Error> for Error {
    fn from(value: ssal::avs::Error) -> Self {
        Self::Ssal(value)
    }
}
