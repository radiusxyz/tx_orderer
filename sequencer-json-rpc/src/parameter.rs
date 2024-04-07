use std::fmt::Debug;

use sequencer_core::{
    async_trait::async_trait,
    error::Error,
    serde::{de::DeserializeOwned, ser::Serialize},
};

/// Defines the necessary traits for a type to be used as an RPC parameter.
///
/// # Examples
/// ```
/// use rpc::RpcParameter;
/// use serde::{Deserialize, Serialize}
///
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct Example {}
///
/// #[async_trait]
/// impl RpcParameter for Example {
///     type Output = String;
///
///     fn method_name() ->  &'static str {
///         ""
///     }
///
///     async fn handler(self) -> Result<Self::Output, Error> {
///         Ok(String::from(""))
///     }
/// }
/// ```
#[async_trait]
pub trait RpcParameter: Debug + DeserializeOwned + Send + Serialize {
    /// The type of the output that the RPC method returns.
    type Output: Debug + DeserializeOwned + Send + Serialize;

    /// Provides the method name for the RPC parameter type.
    ///
    /// # Returns
    ///
    /// A static string slice representing the RPC method's name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assuming a struct MyRpc that implements RpcParameter.
    /// let method = MyRpc::method_name();
    /// println!("RPC Method Name: {}", method);
    /// ```
    fn method_name() -> &'static str;

    /// A handler function to process the RPC request.
    ///
    /// # Returns
    ///
    /// A result containing the desired output on success, or an error on failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assuming a struct MyRpc that implements RpcParameter and an instance `rpc`.
    /// let result = rpc.handler().await;
    /// match result {
    ///     Ok(output) => println!("Output: {:?}", output),
    ///     Err(err) => eprintln!("Error: {:?}", err),
    /// }
    /// ```
    async fn handler(self) -> Result<Self::Output, Error>;
}
