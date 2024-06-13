use jsonrpsee::{core::traits::ToRpcParams, types::Params};
use serde::ser::Serialize;
use serde_json::value::RawValue;

pub type RpcParameter = Params<'static>;

/// Wrapper for the RPC request parameter.
pub(crate) struct Parameter<P>(P)
where
    P: Serialize + Send;

impl<P> From<P> for Parameter<P>
where
    P: Serialize + Send,
{
    fn from(value: P) -> Self {
        Self(value)
    }
}

impl<P> ToRpcParams for Parameter<P>
where
    P: Serialize + Send,
{
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json_string = serde_json::to_string(&self.0)?;
        RawValue::from_string(json_string).map(Some)
    }
}
