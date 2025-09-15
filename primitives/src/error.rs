// LoggerError will be imported from shared crate where needed

#[derive(Debug)]
pub enum Error {
    KvStoreError(radius_sdk::kvstore::KvStoreError),
    Syscall(std::io::Error),
    Config(String), // Simplified to avoid dependency on node
    Logger(String), // Use String instead of LoggerError to avoid circular dependency
    Database(radius_sdk::kvstore::KvStoreError),
    RpcServer(radius_sdk::json_rpc::server::RpcServerError),
    RpcClient(radius_sdk::json_rpc::client::RpcClientError),
    Internal(Box<dyn std::error::Error>),
    Signature(radius_sdk::signature::SignatureError),
    SerializeEthRawTransaction(serde_json::Error),
    LivenessServiceManagerClient(Box<dyn std::error::Error>),
    ValidationServiceManagerClient(Box<dyn std::error::Error>),
    CachedKvStore(radius_sdk::kvstore::CachedKvStoreError),
    DistributedKeyGeneration(String), // Simplified to avoid circular dependency
    RewardManager(String), // Simplified to avoid circular dependency  
    Seeder(String), // Simplified to avoid circular dependency
    Profiler(String), // Simplified to avoid circular dependency
    MerkleTreeDoesNotExist(String),
    InitializeNewCluster(Box<dyn std::error::Error>),
    NoLeader,
    EmptyLeader,
    EmptyLeaderClusterRpcUrl,
    InvalidPlatformBlockHeight,
    ClusterNotFound,
    RollupNotFound,
    SignerNotFound,
    TxOrdererInfoNotFound,
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
    RpcServerTerminated,
    DatabaseVersionMismatch,
    Parse,
    InvalidBatchNumber,
    ClusterMetadataNotFound,
    RollupMetadataNotFound,

    GeneralError(String),

    SyncLeaderTxOrderer,
    InvalidOrderCommitment,
}

unsafe impl Send for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::GeneralError(value)
    }
}

impl From<radius_sdk::kvstore::KvStoreError> for Error {
    fn from(value: radius_sdk::kvstore::KvStoreError) -> Self {
        Self::KvStoreError(value)
    }
}

impl From<radius_sdk::kvstore::CachedKvStoreError> for Error {
    fn from(value: radius_sdk::kvstore::CachedKvStoreError) -> Self {
        Self::CachedKvStore(value)
    }
}

impl From<radius_sdk::signature::SignatureError> for Error {
    fn from(value: radius_sdk::signature::SignatureError) -> Self {
        Self::Signature(value)
    }
}

// ConfigError From implementation removed - ConfigError is now in node

// Logger From implementation removed to avoid circular dependencies

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

// From implementations for client errors removed to avoid circular dependencies
// These will be handled at higher levels in the application

// Profiler From implementation removed to avoid circular dependencies

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Syscall(value)
    }
}
