use crate::value::EdgedbValue;
use crate::Result;
use edgedb_protocol::model::{Json, Uuid};
use edgedb_protocol::value::{self, Value};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
                        v => Err(anyhow::anyhow!("expected {}, got {:?}", stringify!($v), v)),
                    }
                }

                fn to_edgedb_val(self) -> Result<Value> {
                    Ok(Value::$v(self))
                }
            }

            impl EdgedbValue for $t {
                type NativeArgType = $t;

                fn from_edgedb_value(value: Value) -> Result<Self> {
                    <$t>::from_edgedb_val(value)
                }

                // fn to_edgedb_value(self) -> Result<Value> {
                //     <$t>::to_edgedb_val(self)
                // }
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
    String => Str,
    Uuid => Uuid
}

/// Wrapper around your type to pass them as JSON query arguments
pub struct EdgedbJson<T: DeserializeOwned + Serialize>(pub T);

impl<T: DeserializeOwned + Serialize> EdgedbPrim for EdgedbJson<T> {
    fn from_edgedb_val(value: Value) -> Result<Self> {
        if let Value::Json(s) = value {
            Ok(Self(serde_json::from_str(&s)?))
        } else {
            Err(anyhow::anyhow!("expected: {:?}", value))
        }
    }

    fn to_edgedb_val(self) -> Result<Value> {
        let val = serde_json::to_string(&self.0)?;

        // safety: we just serialized this value
        Ok(Value::Json(unsafe { Json::new_unchecked(val) }))
    }
}
