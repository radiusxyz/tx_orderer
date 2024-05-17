use sequencer_core::{
    error::Error,
    hyper::{header, Method},
    jsonrpsee::{
        server::{middleware::http::ProxyGetRequestLayer, Server, ServerHandle},
        RpcModule,
    },
    tower::ServiceBuilder,
    tower_http::cors::{Any, CorsLayer},
    traits::Data,
};

use crate::method::RpcMethod;

pub struct RpcServer {
    rpc_module: RpcModule<()>,
}

impl Default for RpcServer {
    fn default() -> Self {
        Self {
            rpc_module: RpcModule::new(()),
        }
    }
}

impl RpcServer {
    pub fn register_rpc_method<R>(&mut self) -> Result<(), Error>
    where
        R: RpcMethod + Send,
        R::Response: Data + 'static,
    {
        self.rpc_module
            .register_async_method(R::method_name(), |parameter, _state| async move {
                let rpc_parameter: R = parameter.parse().map_err(Error::new)?;
                rpc_parameter.handler().await
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
