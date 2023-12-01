/// todo: local Result type
pub use anyhow::Result;

use edgedb_tokio::Client;
pub use nonempty::{nonempty, NonEmpty};

use edgedb_protocol::{codec::ObjectShape, query_arg::QueryArgs, value::Value};

pub trait EdgedbObject: Sized {
    fn from_edgedb_object(shape: ObjectShape, fields: Vec<Option<Value>>) -> Result<Self>;
    fn to_edgedb_object(&self) -> Result<(ObjectShape, Vec<Option<Value>>)>;
}

pub trait EdgedbValue: Sized {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality;

    fn from_edgedb_value(value: Value) -> Result<Self>;
    fn to_edgedb_value(&self) -> Result<Value>;

    async fn query_direct(client: &Client, q: &str) -> Result<Self>;
}

impl<T: EdgedbObject> EdgedbValue for T {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::One;

    fn from_edgedb_value(value: Value) -> Result<Self> {
        let (shape, fields) = match value {
            Value::Object { shape, fields } => (shape, fields),
            _ => return Err(anyhow::anyhow!("expected object")),
        };
        Self::from_edgedb_object(shape, fields)
    }

    fn to_edgedb_value(&self) -> Result<Value> {
        let (shape, fields) = self.to_edgedb_object()?;
        Ok(Value::Object { shape, fields })
    }

    async fn query_direct(client: &Client, q: &str) -> Result<Self> {
        let val = client.query_required_single::<Value, _>(q, &()).await?;
        let val = Self::from_edgedb_value(val)?;
        Ok(val)
    }
}

impl<T: EdgedbObject> EdgedbValue for Option<T> {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::AtMostOne;

    fn from_edgedb_value(value: Value) -> Result<Self> {
        match value {
            Value::Object { shape, fields } => {
                if fields.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(T::from_edgedb_object(shape, fields)?))
                }
            }
            Value::Nothing => Ok(None),
            _ => Err(anyhow::anyhow!("expected object or None")),
        }
    }

    fn to_edgedb_value(&self) -> Result<Value> {
        match self {
            Some(value) => value.to_edgedb_value(),
            None => Ok(Value::Nothing),
        }
    }

    async fn query_direct(client: &Client, q: &str) -> Result<Self> {
        let val = client.query_single::<Value, _>(q, &()).await?;
        let val = val.map(|val| T::from_edgedb_value(val)).transpose()?;
        Ok(val)
    }
}

impl<T: EdgedbObject> EdgedbValue for Vec<T> {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::Many;

    fn from_edgedb_value(value: Value) -> Result<Self> {
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

    fn to_edgedb_value(&self) -> Result<Value> {
        let vs = self
            .iter()
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
        Ok(val)
    }
}

impl<T: EdgedbObject> EdgedbValue for NonEmpty<T> {
    const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
        edgedb_protocol::server_message::Cardinality::AtLeastOne;

    fn from_edgedb_value(value: Value) -> Result<Self> {
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

    fn to_edgedb_value(&self) -> Result<Value> {
        let vs = self
            .iter()
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
}

pub async fn query<T: EdgedbValue>(client: &Client, q: &str) -> Result<T> {
    let val = T::query_direct(client, q).await?;
    Ok(val)
}

#[cfg(test)]
mod test {
    use crate::query;

    #[tokio::test]
    async fn some_queries() -> anyhow::Result<()> {
        let conn = edgedb_tokio::create_client().await?;

        let v = query::<i64>(&conn, "select 7*8").await?;

        Ok(())
    }
}
