#[derive(Debug)]
pub enum Error {
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Database(database::Error),
    JsonRPC(radius_sequencer_sdk::json_rpc::Error),
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
    InvalidClusterType,
}

unsafe impl Send for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<database::Error> for Error {
    fn from(value: database::Error) -> Self {
        Self::Database(value)
    }
}

impl From<radius_sequencer_sdk::json_rpc::Error> for Error {
    fn from(value: radius_sequencer_sdk::json_rpc::Error) -> Self {
        Self::JsonRPC(value)
    }
}

impl From<ssal::avs::Error> for Error {
    fn from(value: ssal::avs::Error) -> Self {
        Self::Ssal(value)
    }
}
