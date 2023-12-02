use crate::value::EdgedbValue;
use crate::Result;
use edgedb_protocol::model::{Json, Uuid};
use edgedb_protocol::value::{self, Value};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// One of the primitive EdgeDB types, including JSON (see [`EdgedbJson`]). Implement this for your types if they are primitive-convertible.
pub trait EdgedbPrim: Sized {
    const TYPE_CAST: &'static str;

    fn from_edgedb_val(value: Value) -> Result<Self>;

    fn to_edgedb_val(self) -> Result<Value>;
}

macro_rules! impl_prim {
    ($($t:ty => $v:ident $name:literal),*) => {
        $(
            impl EdgedbPrim for $t {
                const TYPE_CAST: &'static str = $name;

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
    i16 => Int16 "int16",
    i32 => Int32 "int32",
    i64 => Int64 "int64",
    f32 => Float32 "float32",
    f64 => Float64 "float64",
    bool => Bool "bool",
    String => Str "str",
    Uuid => Uuid "uuid"
}

/// Wrapper around your serializable types to pass them as JSON query arguments
pub struct EdgedbJson<T: DeserializeOwned + Serialize>(pub T);

impl<T: DeserializeOwned + Serialize> EdgedbPrim for EdgedbJson<T> {
    const TYPE_CAST: &'static str = "json";

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
