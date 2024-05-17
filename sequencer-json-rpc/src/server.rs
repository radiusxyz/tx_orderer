use sequencer_core::{
    error::{Error, WrapError},
    error_context,
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
    rpc_endpoint: Option<String>,
    rpc_module: RpcModule<()>,
}

impl Default for RpcServer {
    fn default() -> Self {
        Self {
            rpc_endpoint: None,
            rpc_module: RpcModule::new(()),
        }
    }
}

impl RpcServer {
    pub fn set_rpc_endpoint(&mut self, rpc_endpoint: impl AsRef<str>) {
        self.rpc_endpoint = Some(rpc_endpoint.as_ref().into());
    }

    pub fn register_rpc_method<R>(&mut self) -> Result<(), Error>
    where
        R: RpcMethod + Send,
        R::Response: Data + 'static,
    {
        self.rpc_module
            .register_async_method(R::method_name(), |parameter, _state| async move {
                let rpc_parameter: R = parameter.parse().wrap(error_context!(parameter))?;
                rpc_parameter.handler().await
            })
            .wrap(error_context!(R::method_name()))
            .unwrap();
        Ok(())
    }

    pub async fn init(self) -> ServerHandle {
        if let Some(rpc_endpoint) = self.rpc_endpoint {
            let cors = CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any)
                .allow_headers([header::CONTENT_TYPE]);

            let middleware = ServiceBuilder::new()
                .layer(cors)
                .layer(ProxyGetRequestLayer::new("/health", "system_health").unwrap());

            let server = Server::builder()
                .set_http_middleware(middleware)
                .build(rpc_endpoint)
                .await
                .unwrap();

            server.start(self.rpc_module)
        } else {
            panic!("RPC endpoint is not set.");
        }
    }
}
