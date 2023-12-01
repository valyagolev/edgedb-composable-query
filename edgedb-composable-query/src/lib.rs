/// todo: local Result type
pub use anyhow::Result;

use edgedb_tokio::Client;
pub use nonempty::{nonempty, NonEmpty};
pub mod prim;
pub mod value;
use crate::value::EdgedbSetValue;
use edgedb_protocol::{codec::ObjectShape, query_arg::QueryArgs, value::Value};
use value::EdgedbValue;

pub trait EdgedbObject: Sized {
    fn from_edgedb_object(shape: ObjectShape, fields: Vec<Option<Value>>) -> Result<Self>;
    fn to_edgedb_object(&self) -> Result<(ObjectShape, Vec<Option<Value>>)>;
}

pub async fn query<T: EdgedbValue>(client: &Client, q: &str) -> Result<T> {
    let val = T::query_direct(client, q).await?;
    Ok(val)
}

#[cfg(test)]
mod test {
    use crate::{
        query,
        value::{EdgedbSetValue, EdgedbValue},
        EdgedbObject,
    };

    #[derive(Debug, PartialEq)]
    struct ExamplImplStruct {
        a: String,
        b: Option<String>,
    }

    impl EdgedbObject for ExamplImplStruct {
        fn from_edgedb_object(
            shape: edgedb_protocol::codec::ObjectShape,
            mut fields: Vec<Option<edgedb_protocol::value::Value>>,
        ) -> anyhow::Result<Self> {
            let mut a = None;
            let mut b = None;

            for (i, s) in shape.elements.iter().enumerate() {
                match s.name.as_str() {
                    "a" => {
                        a = fields[i]
                            .take()
                            .map(|v| EdgedbSetValue::from_edgedb_set_value(v))
                            .transpose()?;
                    }
                    "b" => {
                        b = fields[i]
                            .take()
                            .map(|v| EdgedbSetValue::from_edgedb_set_value(v))
                            .transpose()?;
                    }
                    _ => {}
                }
            }

            Ok(Self {
                a: EdgedbSetValue::interpret_possibly_missing_required_value(a)?,
                b: EdgedbSetValue::interpret_possibly_missing_required_value(b)?,
            })
        }

        fn to_edgedb_object(
            &self,
        ) -> anyhow::Result<(
            edgedb_protocol::codec::ObjectShape,
            Vec<Option<edgedb_protocol::value::Value>>,
        )> {
            todo!()
        }
    }

    #[tokio::test]
    async fn some_queries() -> anyhow::Result<()> {
        let conn = edgedb_tokio::create_client().await?;

        assert_eq!(query::<i64>(&conn, "select 7*8").await?, 56);

        assert_eq!(
            query::<ExamplImplStruct>(&conn, "select {a:='aaa',b:=<str>{}}").await?,
            ExamplImplStruct {
                a: "aaa".to_string(),
                b: None
            }
        );

        assert_eq!(
            query::<ExamplImplStruct>(&conn, "select {a:='aaa',b:=<str>{'cc'}}").await?,
            ExamplImplStruct {
                a: "aaa".to_string(),
                b: Some("cc".to_string())
            }
        );

        assert!(
            query::<ExamplImplStruct>(&conn, "select {a:='aaa',b:=<str>{'cc', 'dd'}}")
                .await
                .is_err()
        );

        Ok(())
    }
}
