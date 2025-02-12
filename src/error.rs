use crate::logger::LoggerError;

#[derive(Debug)]
pub enum Error {
    KvStoreError(radius_sdk::kvstore::KvStoreError),
    Syscall(std::io::Error),
    Config(crate::types::ConfigError),
    Logger(LoggerError),
    Database(radius_sdk::kvstore::KvStoreError),
    RpcServer(radius_sdk::json_rpc::server::RpcServerError),
    RpcClient(radius_sdk::json_rpc::client::RpcClientError),
    Internal(Box<dyn std::error::Error>),
    Signature(radius_sdk::signature::SignatureError),
    SerializeEthRawTransaction(serde_json::Error),
    LivenessClient(Box<dyn std::error::Error>),
    ValidationClient(Box<dyn std::error::Error>),
    CachedKvStore(radius_sdk::kvstore::CachedKvStoreError),
    DistributedKeyGeneration(
        crate::client::liveness_service_manager::distributed_key_generation::DistributedKeyGenerationClientError,
    ),
    Seeder(crate::client::liveness_service_manager::seeder::SeederError),
    Profiler(crate::profiler::ProfilerError),

    MerkleTreeDoesNotExist(String),
    InitializeNewCluster(Box<dyn std::error::Error>),
    EmptyLeader,
    EmptyLeaderClusterRpcUrl,
    InvalidPlatformBlockHeight,
    ClusterNotFound,
    RollupNotFound,
    SignerNotFound,
    SequencerInfoNotFound,
    ExecutorAddressNotFound,
    PlainDataDoesNotExist,
    UnsupportedEncryptedMempool,
    BlockHeightMismatch,
    UnsupportedPlatform,
    UnsupportedValidationServiceProvider,
    UnsupportedRollupType,
    UnsupportedOrderCommitmentType,
    InvalidURL(reqwest::Error),
    HealthCheck(reqwest::Error),
    NotExistRollupMetadata,
    MutexError,
    NoEndpointsAvailable,

    Decryption,
    Deserialize,

    Convert,
    InvalidSignature,
    InvalidTransaction,
    ExceedMaxGasLimit,
    RpcServerTerminated,
    DatabaseVersionMismatch,
    Parse,
}

unsafe impl Send for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<crate::types::ConfigError> for Error {
    fn from(value: crate::types::ConfigError) -> Self {
        Self::Config(value)
    }
}

impl From<crate::logger::LoggerError> for Error {
    fn from(value: crate::logger::LoggerError) -> Self {
        Self::Logger(value)
    }
}

impl From<radius_sdk::json_rpc::server::RpcServerError> for Error {
    fn from(value: radius_sdk::json_rpc::server::RpcServerError) -> Self {
        Self::RpcServer(value)
    }
}

impl From<radius_sdk::json_rpc::client::RpcClientError> for Error {
    fn from(value: radius_sdk::json_rpc::client::RpcClientError) -> Self {
        Self::RpcClient(value)
    }
}

impl From<crate::client::liveness_service_manager::distributed_key_generation::DistributedKeyGenerationClientError>
    for Error
{
    fn from(
        value: crate::client::liveness_service_manager::distributed_key_generation::DistributedKeyGenerationClientError,
    ) -> Self {
        Self::DistributedKeyGeneration(value)
    }
}

impl From<crate::client::liveness_service_manager::seeder::SeederError> for Error {
    fn from(value: crate::client::liveness_service_manager::seeder::SeederError) -> Self {
        Self::Seeder(value)
    }
}

impl From<crate::profiler::ProfilerError> for Error {
    fn from(value: crate::profiler::ProfilerError) -> Self {
        Self::Profiler(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Syscall(value)
    }
}
