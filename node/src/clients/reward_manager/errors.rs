#[derive(Debug)]
pub enum RewardManagerError {
    Initialize(radius_sdk::json_rpc::client::RpcClientError),
    Register(radius_sdk::json_rpc::client::RpcClientError),
}

impl std::fmt::Display for RewardManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RewardManagerError {}
