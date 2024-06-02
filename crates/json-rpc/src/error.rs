#[derive(Debug)]
pub enum Error {
    BuildClient(jsonrpsee::core::ClientError),
    RpcRequest(jsonrpsee::core::ClientError),
    RegisterRpcMethod(jsonrpsee::core::RegisterMethodError),
    ParseParameter(jsonrpsee::types::error::ErrorObjectOwned),
    RpcMiddleware(jsonrpsee::server::middleware::http::InvalidPath),
    BuildServer(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct RpcError(Box<dyn std::error::Error>);

impl<E> From<E> for RpcError
where
    E: std::error::Error + 'static,
{
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}

impl Into<String> for RpcError {
    fn into(self) -> String {
        self.0.to_string()
    }
}
