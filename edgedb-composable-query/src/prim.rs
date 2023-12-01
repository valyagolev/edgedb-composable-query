use crate::value::EdgedbValue;
use crate::Result;
use edgedb_protocol::value::Value;
use edgedb_tokio::Client;

pub trait EdgedbPrim: Sized {
    fn from_edgedb_val(value: Value) -> Result<Self>;

    fn to_edgedb_val(self) -> Result<Value>;
}

macro_rules! impl_prim {
    ($($t:ty => $v:ident),*) => {
        $(
            impl EdgedbPrim for $t {
                fn from_edgedb_val(value: Value) -> Result<Self> {
                    match value {
                        Value::$v(v) => Ok(v),
                        _ => Err(anyhow::anyhow!("expected {}", stringify!($v))),
                    }
                }

                fn to_edgedb_val(self) -> Result<Value> {
                    Ok(Value::$v(self))
                }
            }

            impl EdgedbValue for $t {
                const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
                    edgedb_protocol::server_message::Cardinality::One;

                fn from_edgedb_value(value: Value) -> Result<Self> {
                    <$t>::from_edgedb_val(value)
                }

                fn to_edgedb_value(self) -> Result<Value> {
                    <$t>::to_edgedb_val(self)
                }

                async fn query_direct(client: &Client, q: &str) -> Result<Self> {
                    let val = client.query_required_single::<Value, _>(q, &()).await?;
                    let val = Self::from_edgedb_value(val)?;
                    Ok(val)
                }
            }
        )*
    };
    () => {

    };
}

impl_prim! {
    i16 => Int16,
    i32 => Int32,
    i64 => Int64,
    f32 => Float32,
    f64 => Float64,
    bool => Bool,
    String => Str
}
