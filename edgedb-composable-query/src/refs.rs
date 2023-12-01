use edgedb_protocol::{model::Uuid, value::Value};
use itertools::Itertools;

use crate::{prim::EdgedbPrim, value::EdgedbValue, EdgedbObject};

#[derive(Debug, PartialEq, Eq)]
pub struct Ref<T: EdgedbObject> {
    id: Uuid,
    known_value: Option<T>,
}

impl<T: EdgedbObject> EdgedbValue for Ref<T> {
    type NativeArgType = Value;

    fn from_edgedb_value(value: edgedb_protocol::value::Value) -> anyhow::Result<Self> {
        match value {
            edgedb_protocol::value::Value::Object { shape, mut fields } => {
                let uuid_i = shape.elements.iter().find_position(|e| e.name == "id").map(|(i, _)| i).ok_or_else(|| anyhow::anyhow!("Expected an object with an 'id' field when deserializing a Ref, got something else"))?;

                let id = Uuid::from_edgedb_val(fields[uuid_i].take().ok_or_else(|| anyhow::anyhow!("Expected an object with an 'id' field when deserializing a Ref, got something else"))?)?;

                let known_value = if shape.elements.len() != 1 {
                    Some(T::from_edgedb_object(shape, fields)?)
                } else {
                    None
                };

                Ok(Self { id, known_value })
            }
            edgedb_protocol::value::Value::Uuid(_) => {
                anyhow::bail!("Expected an object when deserializing a Ref, got uuid (do we want to handle this?)")
            }
            edgedb_protocol::value::Value::SparseObject(_) => {
                anyhow::bail!("Expected an object when deserializing a Ref, got sparse object (?)")
            }
            _ => {
                anyhow::bail!("Expected an object when deserializing a Ref, got something else")
            }
        }
    }

    // fn to_edgedb_value(self) -> anyhow::Result<edgedb_protocol::value::Value> {
    //     todo!()
    // }
}

#[cfg(test)]
mod test {
    use crate::{value::EdgedbSetValue, EdgedbObject};

    #[derive(Debug, PartialEq)]
    struct Inner {
        req: String,
        opt: Option<String>,
    }

    impl EdgedbObject for Inner {
        fn from_edgedb_object(
            shape: edgedb_protocol::codec::ObjectShape,
            mut fields: Vec<Option<edgedb_protocol::value::Value>>,
        ) -> anyhow::Result<Self> {
            let mut req = None;
            let mut opt = None;

            for (i, s) in shape.elements.iter().enumerate() {
                match s.name.as_str() {
                    "req" => {
                        req = fields[i]
                            .take()
                            .map(EdgedbSetValue::from_edgedb_set_value)
                            .transpose()?;
                    }
                    "opt" => {
                        opt = fields[i]
                            .take()
                            .map(EdgedbSetValue::from_edgedb_set_value)
                            .transpose()?;
                    }
                    _ => {}
                }
            }

            Ok(Self {
                req: EdgedbSetValue::interpret_possibly_missing_required_value(req)?,
                opt: EdgedbSetValue::interpret_possibly_missing_required_value(opt)?,
            })
        }

        // fn to_edgedb_object(
        //     &self,
        // ) -> anyhow::Result<(
        //     edgedb_protocol::codec::ObjectShape,
        //     Vec<Option<edgedb_protocol::value::Value>>,
        // )> {
        //     todo!()
        // }
    }

    #[tokio::test]
    async fn some_queries() -> anyhow::Result<()> {
        let _conn = edgedb_tokio::create_client().await?;

        // dbg!(query::<Vec<Ref<Inner>>>(&conn, "select Inner;").await?);
        // dbg!(query::<Vec<Ref<Inner>>>(&conn, "select Inner {id, opt, req};").await?);

        Ok(())
    }
}
