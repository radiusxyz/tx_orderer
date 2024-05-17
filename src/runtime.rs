use std::{
    future::Future,
    mem::MaybeUninit,
    path::{Path, PathBuf},
    sync::Once,
};

use sequencer_core::{
    error::Error,
    jsonrpsee::server::ServerHandle,
    tokio::{self, runtime::Runtime, task::JoinHandle},
    tracing_subscriber,
    traits::Data,
};
use sequencer_database::client::Database;
use sequencer_json_rpc::{method::RpcMethod, server::RpcServer};

static mut RUNTIME: MaybeUninit<SequencerRuntime> = MaybeUninit::uninit();
static INIT: Once = Once::new();

pub fn runtime() -> &'static SequencerRuntime {
    if INIT.is_completed() {
        unsafe { RUNTIME.assume_init_ref() }
    } else {
        panic!("Runtime has not been initialized.");
    }
}

pub struct SequencerBuilder {
    pub database_path: Option<PathBuf>,
    pub rpc_endpoint: Option<String>,
    pub rpc_server: RpcServer,
}

impl Default for SequencerBuilder {
    fn default() -> Self {
        Self {
            database_path: None,
            rpc_endpoint: None,
            rpc_server: RpcServer::default(),
        }
    }
}

impl SequencerBuilder {
    pub fn set_database_path(mut self, path: impl AsRef<Path>) -> Self {
        self.database_path = Some(path.as_ref().into());
        self
    }

    pub fn set_rpc_endpoint(mut self, endpoint: impl AsRef<str>) -> Self {
        self.rpc_endpoint = Some(endpoint.as_ref().into());
        self
    }

    pub fn register_rpc_method<R>(mut self) -> Result<Self, Error>
    where
        R: RpcMethod + Send,
        R::Response: Data + 'static,
    {
        self.rpc_server.register_rpc_method::<R>()?;
        Ok(self)
    }

    pub fn build(self) -> Result<SequencerRuntime, Error> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(Error::new)?;

        let database = match &self.database_path {
            Some(database_path) => Ok(Database::new(database_path)?),
            None => Err("Database path is not set."),
        }?;

        let server_handle = match &self.rpc_endpoint {
            Some(rpc_endpoint) => Ok(runtime.block_on(self.rpc_server.init(rpc_endpoint))?),
            None => Err("RPC endpoint is not set."),
        }?;

        Ok(SequencerRuntime {
            database,
            server_handle,
            runtime,
        })
    }
}

#[allow(unused)]
pub struct SequencerRuntime {
    database: Database,
    server_handle: ServerHandle,
    runtime: Runtime,
}

impl SequencerRuntime {
    pub fn init(self) {
        unsafe {
            INIT.call_once(|| {
                tracing_subscriber::fmt().init();
                RUNTIME.write(self);
            });
        }
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }
}
