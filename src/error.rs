#[derive(Debug)]
pub enum Error {
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Database(radius_sdk::kvstore::KvStoreError),
    RpcError(radius_sdk::json_rpc::Error),
    Signature(radius_sdk::signature::SignatureError),
    Deserialize(serde_json::Error),
    CreateLivenessClient(Box<dyn std::error::Error>),
    InitializeLivenessClient(Box<dyn std::error::Error>),
    InitializeValidationClient(Box<dyn std::error::Error>),
    CachedKvStore(radius_sdk::kvstore::CachedKvStoreError),
    Uninitialized,
    EmptySequencerList,
    LeaderIndexOutOfBound,
    EmptyLeaderRpcUrl,
    FetchResponse,
    ClusterDown,
    InvalidSequencerPort,
    InvalidBlockHeight,
    InvalidLeaderIndex,

    LoadConfigOption(std::io::Error),
    ParseTomlString(toml::de::Error),

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

    NotSupportedPlatform,
    NotSupportedValidationServiceProvider,
    NotSupportedRollupType,

    NotExistPlainData,
    NotSupportEncryptedMempool,
    BlockHeightMismatch,

    NotSupportedOrderCommitmentType,
}

unsafe impl Send for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<radius_sdk::json_rpc::Error> for Error {
    fn from(value: radius_sdk::json_rpc::Error) -> Self {
        Self::RpcError(value)
    }
}
