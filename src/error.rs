use crate::logger::LoggerError;

#[derive(Debug)]
pub enum Error {
    Config(crate::types::ConfigError),
    Database(radius_sdk::kvstore::KvStoreError),
    LoggerError(LoggerError),
    RpcServer(radius_sdk::json_rpc::server::RpcServerError),
    Signature(radius_sdk::signature::SignatureError),
    SerializeEthRawTransaction(serde_json::Error),
    CreateLivenessClient(Box<dyn std::error::Error>),
    InitializeLivenessClient(Box<dyn std::error::Error>),
    InitializeValidationClient(Box<dyn std::error::Error>),
    CachedKvStore(radius_sdk::kvstore::CachedKvStoreError),
    DistributedKeyGeneration(
        crate::client::liveness::distributed_key_generation::DistributedKeyGenerationClientError,
    ),
    Seeder(crate::client::liveness::seeder::SeederError),

    EmptyLeader,
    EmptyLeaderClusterRpcUrl,
    InvalidPlatformBlockHeight,
    ClusterNotFound,
    ExecutorAddressNotFound,
    PlainDataDoesNotExist,
    UnsupportedEncryptedMempool,
    BlockHeightMismatch,

    UnsupportedPlatform,
    UnsupportedValidationServiceProvider,
    UnsupportedRollupType,
    UnsupportedOrderCommitmentType,
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

impl From<radius_sdk::json_rpc::server::RpcServerError> for Error {
    fn from(value: radius_sdk::json_rpc::server::RpcServerError) -> Self {
        Self::RpcServer(value)
    }
}

impl From<crate::client::liveness::distributed_key_generation::DistributedKeyGenerationClientError>
    for Error
{
    fn from(
        value: crate::client::liveness::distributed_key_generation::DistributedKeyGenerationClientError,
    ) -> Self {
        Self::DistributedKeyGeneration(value)
    }
}

impl From<crate::client::liveness::seeder::SeederError> for Error {
    fn from(value: crate::client::liveness::seeder::SeederError) -> Self {
        Self::Seeder(value)
    }
}
