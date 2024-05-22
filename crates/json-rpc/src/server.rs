use std::fmt::Debug;

use database::client::Database;
use primitives::{
    error::Error,
    hyper::{header, Method},
    jsonrpsee::{
        server::{middleware::http::ProxyGetRequestLayer, Server, ServerHandle},
        RpcModule,
    },
    serde::{de::DeserializeOwned, ser::Serialize},
    tower::ServiceBuilder,
    tower_http::cors::{Any, CorsLayer},
};

use crate::method::RpcMethod;

pub struct RpcServer {
    rpc_module: RpcModule<Database>,
}

impl RpcServer {
    pub fn new(context: Database) -> Self {
        Self {
            rpc_module: RpcModule::new(context),
        }
    }

    pub fn register_rpc_method<R>(&mut self) -> Result<(), Error>
    where
        R: RpcMethod + Send,
        R::Response: Clone + Debug + DeserializeOwned + Serialize + 'static,
    {
        self.rpc_module
            .register_async_method(R::method_name(), |parameter, state| async move {
                let rpc_parameter: R = parameter.parse().map_err(Error::new)?;
                rpc_parameter.handler(state).await
            })
            .map_err(Error::new)?;
        Ok(())
    }

    pub async fn init(self, rpc_endpoint: impl AsRef<str>) -> Result<ServerHandle, Error> {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers([header::CONTENT_TYPE]);

        let middleware = ServiceBuilder::new()
            .layer(cors)
            .layer(ProxyGetRequestLayer::new("/health", "system_health").unwrap());

        let server = Server::builder()
            .set_http_middleware(middleware)
            .build(rpc_endpoint.as_ref())
            .await
            .map_err(Error::new)?;

        Ok(server.start(self.rpc_module))
    }
}
