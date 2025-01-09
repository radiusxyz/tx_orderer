use std::sync::Arc;

use pyroscope::{pyroscope::PyroscopeAgentRunning, PyroscopeAgent, PyroscopeError};
use pyroscope_pprofrs::{pprof_backend, PprofConfig};
use tokio::sync::Mutex;

pub struct Profiler {
    inner: Arc<Mutex<ProfilerInner>>,
}

struct ProfilerInner {
    agent: Option<PyroscopeAgent<PyroscopeAgentRunning>>,
}

impl Drop for ProfilerInner {
    fn drop(&mut self) {
        if let Some(agent) = self.agent.take() {
            let agent_ready = agent.stop().map_err(ProfilerError::Stop).unwrap();
            agent_ready.shutdown();
        }
    }
}

impl From<PyroscopeAgent<PyroscopeAgentRunning>> for ProfilerInner {
    fn from(value: PyroscopeAgent<PyroscopeAgentRunning>) -> Self {
        Self { agent: Some(value) }
    }
}

impl Clone for Profiler {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Profiler {
    /// Initialize the profiler agent.
    ///
    /// # Parameters
    ///
    /// - `server_url`: The explorer URL for the dashboard.
    /// - `application_name`: The application name that you are running the
    ///   profiler in.
    /// - `sample_rate`: Sampling frequency in Hertz.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Initialize the profiler server at "http://127.0.0.1:4040" for the "sequencer" with sampling rate of 100 Hertz.
    /// let profiler = Profiler::init("http://127.0.0.1:4040", "sequencer", 100)?;
    /// ```
    pub fn init(
        server_url: &str,
        application_name: &str,
        sample_rate: u32,
    ) -> Result<Self, ProfilerError> {
        let agent = PyroscopeAgent::builder(server_url, application_name)
            .backend(pprof_backend(PprofConfig::new().sample_rate(sample_rate)))
            .build()
            .map_err(ProfilerError::Initialize)?;

        let agent_running = agent.start().map_err(ProfilerError::Start)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(agent_running.into())),
        })
    }

    /// Use tag wrapper to scope the function(s) you want to measure
    /// the performance of.
    ///
    /// # Examples
    ///
    /// Using the profiler inside the RPC handler function.
    ///
    /// ```rust
    /// pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
    ///     let parameter = parameter.parse::<Self>()?;
    ///
    ///     let (start, end) = context.profiler().tag_wrapper().await?;
    ///
    ///     start("send_encrypted_transaction", "arbitrary_function_1");
    ///     arbitrary_function_1();
    ///     end("send_encrypted_transaction", "arbitrary_function_1");
    ///
    ///     start("send_encrypted_transaction", "arbitrary_function_2");
    ///     arbitrary_function_2();
    ///     end("send_encrypted_transaction", "arbitrary_function_2");
    /// }
    /// ```
    ///
    /// You may group multiple functions with the same tag.
    ///
    /// ```rust
    /// pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
    ///     let parameter = parameter.parse::<Self>()?;
    ///
    ///     let (start, end) = context.profiler().tag_wrapper().await?;
    ///
    ///     start("send_encrypted_transaction", "multiple functions");
    ///     arbitrary_function_1();
    ///     arbitrary_function_2();
    ///     end("send_encrypted_transaction", "multiple functions");
    /// }
    /// ```
    pub async fn tag_wrapper(
        &self,
    ) -> Result<
        (
            impl Fn(String, String) -> Result<(), PyroscopeError>,
            impl Fn(String, String) -> Result<(), PyroscopeError>,
        ),
        ProfilerError,
    > {
        let profiler = self.inner.lock().await;

        if let Some(agent) = profiler.agent.as_ref() {
            Ok(agent.tag_wrapper())
        } else {
            Err(ProfilerError::TagWrapper)
        }
    }
}

#[derive(Debug)]
pub enum ProfilerError {
    Initialize(PyroscopeError),
    Start(PyroscopeError),
    Stop(PyroscopeError),
    TagWrapper,
}

impl std::fmt::Display for ProfilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ProfilerError {}
