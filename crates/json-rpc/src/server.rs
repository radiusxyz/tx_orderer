use std::{future::Future, sync::Arc};

use hyper::{header, Method};
use jsonrpsee::{
    server::{middleware::http::ProxyGetRequestLayer, Server, ServerHandle},
    IntoResponse, RpcModule,
};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{types::RpcParameter, Error, ErrorKind};

pub struct RpcServer<C>
where
    C: Send + Sync + 'static,
{
    rpc_module: RpcModule<C>,
}

impl<C> RpcServer<C>
where
    C: Send + Sync + 'static,
{
    pub fn new(context: C) -> Self {
        Self {
            rpc_module: RpcModule::new(context),
        }
    }

    pub fn register_rpc_method<H, F, R>(
        mut self,
        method: &'static str,
        handler: H,
    ) -> Result<Self, Error>
    where
        H: Fn(RpcParameter, Arc<C>) -> F + Clone + Send + Sync + 'static,
        F: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.rpc_module
            .register_async_method(method, handler)
            .map_err(|error| (ErrorKind::RegisterRpcMethod, error))?;

        Ok(self)
    }

    pub async fn init(self, rpc_url: impl AsRef<str>) -> Result<ServerHandle, Error> {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers([header::CONTENT_TYPE]);

        let middleware = ServiceBuilder::new().layer(cors).layer(
            ProxyGetRequestLayer::new("/health", "health")
                .map_err(|error| (ErrorKind::RpcMiddleware, error))?,
        );

        let server = Server::builder()
            .set_http_middleware(middleware)
            .build(rpc_url.as_ref())
            .await
            .map_err(|error| (ErrorKind::BuildServer, error))?;

        Ok(server.start(self.rpc_module))
    }
}
