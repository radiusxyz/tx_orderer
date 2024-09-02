#[derive(Debug)]
pub enum Error {
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Database(radius_sequencer_sdk::kvstore::KvStoreError),
    RpcError(radius_sequencer_sdk::json_rpc::Error),
    Deserialize(serde_json::Error),
    InitializeLivenessClient(Box<dyn std::error::Error>),
    Uninitialized,
    EmptySequencerList,
    LeaderIndexOutOfBound,
    EmptyLeaderRpcUrl,
    FetchResponse,
    ClusterDown,
    InvalidSequencerPort,
    InvalidBlockHeight,

    LoadConfigOption,
    ParseTomlString,

    RemoveConfigDirectory,
    CreateConfigDirectory,
    CreateConfigFile,
    CreatePrivateKeyFile,

    RegisterRpcUrl,
    GetSequencerRpcUrlList,

    GetSequencingInfo,
    GetRollupMetadata,

    PvdeZkpInvalid,
    TryDecryptRawTransaction,

    NotfoundRpcUrl,
    NotFoundRollupState,
    NotFoundCluster,
    NotFoundClusterId,
    NotFoundSequencingInfo,

    InvalidTransactionOrder,

    OtherSequencerRpcClientsCountNotCorrect,
}

unsafe impl Send for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
