use std::any;

use sequencer_core::{
    caller,
    error::{Error, WrapError},
    hyper::{header, Method},
    jsonrpsee::{
        server::{middleware::http::ProxyGetRequestLayer, Server, ServerHandle},
        types::ErrorObjectOwned,
        RpcModule,
    },
    serde::ser::Serialize,
    serde_json,
    tokio::net::ToSocketAddrs,
    tower,
    tower_http::cors::{Any, CorsLayer},
};

use crate::parameter::RpcParameter;

/// Represents an RPC server builder with a specified context.
pub struct RpcServerBuilder<C> {
    module: RpcModule<C>,
}

impl<C: Send + Sync + 'static> RpcServerBuilder<C> {
    /// Constructs a new `RpcServerBuilder` with a given context.
    ///
    /// # Arguments
    ///
    /// * `context` - The context associated with the RPC server builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assuming `SomeContext` type is defined elsewhere.
    /// use rpc::RpcServerBuilder;
    ///
    /// let builder = RpcServerBuilder::new(SomeContext::default());
    /// ```
    pub fn new(context: C) -> Self {
        Self {
            module: RpcModule::new(context),
        }
    }

    /// Registers an RPC parameter to the server builder module.
    ///
    /// # Arguments
    ///
    /// * `P` - The RPC parameter type.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the method registration fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assuming `SomeRpcParam` type is defined and implements `RpcParameter`.
    /// use rpc::RpcServerBuilder;
    ///
    /// let mut builder = RpcServerBuilder::new(SomeContext::default())
    ///     .register::<SomeRpcParam1>().unwrap()
    ///     .register::<SomeRpcParam2>().unwrap()
    ///     .register::<SomeRpcParam3>().unwrap();
    /// ```
    pub fn register<P>(&mut self) -> Result<(), Error>
    where
        P: RpcParameter,
        P::Output: Clone + Serialize + 'static,
    {
        self.module
            .register_async_method(P::method_name(), |parameter, _state| async move {
                let rpc_parameter = parameter.parse::<P>()?;
                rpc_parameter
                    .handler()
                    .await
                    .map_err(ErrorObjectOwned::from)
            })
            .wrap_context(
                caller!(RpcServerBuilder::register()),
                format_args!("RPC parameter: {:?}", any::type_name::<P>()),
            )?;
        Ok(())
    }

    /// Builds and initializes an `RpcServer` based on the server builder configuration.
    ///
    /// # Arguments
    ///
    /// * `address` - The socket address to bind the server to.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the server fails to initialize or bind.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rpc::RpcServerBuilder;
    ///
    /// let builder = RpcServerBuilder::new(());
    ///
    /// // Assuming usage within an async block.
    /// let server = builder.build("127.0.0.1:8080").await.unwrap();
    /// ```
    pub async fn build(mut self, address: impl ToSocketAddrs) -> Result<RpcServer, Error> {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers([header::CONTENT_TYPE]);

        let middleware = tower::ServiceBuilder::new().layer(cors).layer(
            ProxyGetRequestLayer::new("/health", "system_health").wrap_context(
                caller!(RpcServerBuilder::build()),
                "Failed to build ProxyGetRequestLayer",
            )?,
        );

        let server = Server::builder()
            .set_http_middleware(middleware)
            .build(address)
            .await
            .wrap(caller!(RpcServerBuilder::build()))?;

        self.module
            .register_method(
                "system_health",
                |_, _| serde_json::json!({ "health": true }),
            )
            .wrap_context(
                caller!(RpcServerBuilder::build()),
                "Failed to register the health checker",
            )?;
        let handle = server.start(self.module);
        Ok(RpcServer(Some(handle)))
    }
}

/// Represents the main RPC server with an optional server handle.
pub struct RpcServer(Option<ServerHandle>);

impl RpcServer {
    pub async fn join(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.stopped().await
        }
    }
}
