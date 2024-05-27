use std::{fmt::Debug, mem::MaybeUninit, sync::Once, thread};

use database::client::Database;
use json_rpc::{
    method::RpcMethod,
    server::{RpcServer, ServerHandle},
};
use primitives::{
    error::Error,
    serde::{de::DeserializeOwned, ser::Serialize},
    tracing_subscriber,
};
use tokio::runtime::{Builder, Runtime};

static mut SERVER: MaybeUninit<Server> = MaybeUninit::uninit();
static mut INIT: Once = Once::new();

pub struct ServerBuilder {
    io_threads: usize,
    decryption_threads: usize,
    database_path: Option<String>,
    rpc_endpoint: Option<String>,
    rpc_server: RpcServer,
    ssal_endpoint: Option<String>,
}

impl Default for ServerBuilder {
    fn default() -> Self {
        // # Safety
        // It is safe to use `unwrap()` because the program is not supposed to start if
        // it fails to fetch the number of available threads.
        let total_thread = thread::available_parallelism().unwrap().get();

        Self {
            io_threads: 2,
            decryption_threads: total_thread - 2,
            database_path: None,
            rpc_endpoint: None,
            rpc_server: RpcServer::new(),
            ssal_endpoint: None,
        }
    }
}

impl ServerBuilder {
    /// Set the number of IO threads.
    pub fn set_io_threads(&mut self, value: usize) -> &mut Self {
        self.io_threads = value;
        self
    }

    /// Set the number of threads for the decryption (blocking operation).
    pub fn set_decryption_threads(&mut self, value: usize) -> &mut Self {
        self.decryption_threads = value;
        self
    }

    /// Set the database path for the server.
    pub fn set_database_path(&mut self, path: impl AsRef<str>) -> &mut Self {
        self.database_path = Some(path.as_ref().into());
        self
    }

    /// Set JSON-RPC endpoint for the server.
    pub fn set_rpc_endpoint(&mut self, endpoint: impl AsRef<str>) -> &mut Self {
        self.rpc_endpoint = Some(endpoint.as_ref().into());
        self
    }

    /// Set SSAL endpoint for the server.
    pub fn set_ssal_endpoint(&mut self, endpoint: impl AsRef<str>) -> &mut Self {
        self.ssal_endpoint = Some(endpoint.as_ref().into());
        self
    }

    /// Register an RPC method.
    pub fn register_rpc_method<R>(&mut self) -> Result<&mut Self, Error>
    where
        R: RpcMethod + Send,
        R::Response: Clone + Debug + DeserializeOwned + Serialize + 'static,
    {
        self.rpc_server.register_rpc_method::<R>()?;
        Ok(self)
    }

    pub fn build(self) -> Result<Server, Error> {
        let database = self.init_database()?;
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(self.io_threads)
            .max_blocking_threads(self.decryption_threads)
            .build()
            .map_err(Error::new)?;
        let rpc_server_handle = match self.rpc_endpoint {
            Some(rpc_endpoint) => runtime.block_on(self.rpc_server.init(rpc_endpoint))?,
            None => return Err(Error::from("Set the RPC endpoint.")),
        };
        Ok(Server {
            database,
            runtime,
            rpc_server_handle,
        })
    }

    fn init_database(&self) -> Result<Database, Error> {
        match &self.database_path {
            Some(path) => Ok(Database::new(path)?),
            None => Err(Error::from("Set the database path.")),
        }
    }
}

#[allow(unused)]
pub struct Server {
    database: Database,
    runtime: Runtime,
    rpc_server_handle: ServerHandle,
}

impl Server {
    pub fn init(self) -> Result<(), Error> {
        unsafe {
            INIT.call_once(|| {
                tracing_subscriber::fmt().init();
                SERVER.write(self);
            });
        }
        Ok(())
    }

    pub fn database(&self) -> Database {
        self.database.clone()
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
}
