pub use crate::Result;
use crate::{prim::EdgedbPrim, EdgedbObject};

use edgedb_tokio::Client;
pub use nonempty::{nonempty, NonEmpty};

use edgedb_protocol::{codec::ObjectShape, query_arg::QueryArgs, value::Value};

pub trait EdgedbValue: Sized {
    fn from_edgedb_value(value: Value) -> Result<Self>;
    fn to_edgedb_value(self) -> Result<Value>;
}

pub trait EdgedbSetValue: Sized {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality;

    fn from_edgedb_set_value(value: Value) -> Result<Self>;
    fn to_edgedb_set_value(self) -> Result<Value>;

    fn interpret_possibly_missing_required_value(val: Option<Self>) -> Result<Self>;

    async fn query_direct(client: &Client, q: &str) -> Result<Self>;
}

impl<T: EdgedbObject> EdgedbValue for T {
    fn from_edgedb_value(value: Value) -> Result<Self> {
        let (shape, fields) = match value {
            Value::Object { shape, fields } => (shape, fields),
            _ => return Err(anyhow::anyhow!("expected object")),
        };
        Self::from_edgedb_object(shape, fields)
    }

    fn to_edgedb_value(self) -> Result<Value> {
        let (shape, fields) = self.to_edgedb_object()?;
        Ok(Value::Object { shape, fields })
    }
}

impl<T: EdgedbValue> EdgedbSetValue for T {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::One;

    async fn query_direct(client: &Client, q: &str) -> Result<Self> {
        let val = client.query_required_single::<Value, _>(q, &()).await?;
        let val = Self::from_edgedb_value(val)?;
        Ok(val)
    }

    fn from_edgedb_set_value(value: Value) -> Result<Self> {
        T::from_edgedb_value(value)
    }

    fn to_edgedb_set_value(self) -> Result<Value> {
        T::to_edgedb_value(self)
    }

    fn interpret_possibly_missing_required_value(val: Option<Self>) -> Result<Self> {
        match val {
            Some(val) => Ok(val),
            None => Err(anyhow::anyhow!("expected single value")),
        }
    }
}

impl<T: EdgedbValue> EdgedbSetValue for Option<T> {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::AtMostOne;

    async fn query_direct(client: &Client, q: &str) -> Result<Self> {
        let val = client.query_single::<Value, _>(q, &()).await?;
        let val = val.map(|val| T::from_edgedb_value(val)).transpose()?;
        Ok(val)
    }

    fn from_edgedb_set_value(value: Value) -> Result<Self> {
        match value {
            Value::Nothing => Ok(None),
            _ => Ok(Some(T::from_edgedb_value(value)?)),
        }
    }

    fn to_edgedb_set_value(self) -> Result<Value> {
        match self {
            Some(v) => T::to_edgedb_value(v),
            None => Ok(Value::Nothing),
        }
    }

    fn interpret_possibly_missing_required_value(val: Option<Self>) -> Result<Self> {
        Ok(val.flatten())
    }
}

impl<T: EdgedbValue> EdgedbSetValue for Vec<T> {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::Many;

    fn from_edgedb_set_value(value: Value) -> Result<Self> {
        match value {
            Value::Nothing => {
                // Ok(Vec::new())
                todo!("Wrong cardinality/type (nothing), or just fine?..")
            }
            Value::Set(vals) => vals
                .into_iter()
                .map(|val| T::from_edgedb_value(val))
                .collect(),
            Value::Array(vals) => {
                todo!("Wrong cardinality/type (array), or just fine?..")
            }
            Value::Object { shape, fields } => {
                todo!("Wrong cardinality/type (object), or just fine?..")
            }
            _ => return Err(anyhow::anyhow!("expected object")),
        }
    }

    fn to_edgedb_set_value(self) -> Result<Value> {
        let vs = self
            .into_iter()
            .map(|v| v.to_edgedb_value())
            .collect::<Result<_>>()?;

        Ok(Value::Set(vs))
    }

    async fn query_direct(client: &Client, q: &str) -> Result<Self> {
        let val = client.query::<Value, _>(q, &()).await?;

        dbg!(&val);

        let val = val
            .into_iter()
            .map(|val| T::from_edgedb_value(val))
            .collect::<Result<_>>()?;

        Ok(val)
    }

    fn interpret_possibly_missing_required_value(val: Option<Self>) -> Result<Self> {
        Ok(val.unwrap_or_default())
    }
}

impl<T: EdgedbValue> EdgedbSetValue for NonEmpty<T> {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::AtLeastOne;

    fn from_edgedb_set_value(value: Value) -> Result<Self> {
        match value {
            Value::Nothing => {
                todo!("NonEmpty: Wrong cardinality/type (nothing), or just fine?..")
            }
            Value::Set(vals) => {
                let vs = vals
                    .into_iter()
                    .map(|val| T::from_edgedb_value(val))
                    .collect::<Result<_>>()?;

                NonEmpty::from_vec(vs).ok_or_else(|| anyhow::anyhow!("expected non-empty set"))
            }
            Value::Array(vals) => {
                todo!("NonEmpty: Wrong cardinality/type (array), or just fine?..")
            }
            Value::Object { shape, fields } => {
                todo!("NonEmpty: Wrong cardinality/type (object), or just fine?..")
            }
            _ => return Err(anyhow::anyhow!("expected object")),
        }
    }

    fn to_edgedb_set_value(self) -> Result<Value> {
        let vs = self
            .into_iter()
            .map(|v| v.to_edgedb_value())
            .collect::<Result<_>>()?;

        Ok(Value::Set(vs))
    }

    async fn query_direct(client: &Client, q: &str) -> Result<Self> {
        let val = client.query::<Value, _>(q, &()).await?;
        let val = val
            .into_iter()
            .map(|val| T::from_edgedb_value(val))
            .collect::<Result<_>>()?;
        NonEmpty::from_vec(val).ok_or_else(|| anyhow::anyhow!("expected non-empty set"))
    }

    fn interpret_possibly_missing_required_value(val: Option<Self>) -> Result<Self> {
        Ok(val.ok_or_else(|| anyhow::anyhow!("expected non-empty set"))?)
    }
}
