/// todo: local Result type
pub use anyhow::Result;

use args::EdgedbQueryArgs;
use edgedb_tokio::Client;
pub use nonempty::{nonempty, NonEmpty};

pub mod args;
pub mod prim;
pub mod refs;
pub mod tuples;
pub mod value;

use crate::value::EdgedbSetValue;
use edgedb_protocol::{codec::ObjectShape, value::Value};
use value::EdgedbValue;

pub trait EdgedbObject: Sized {
    fn from_edgedb_object(shape: ObjectShape, fields: Vec<Option<Value>>) -> Result<Self>;
    fn to_edgedb_object(&self) -> Result<(ObjectShape, Vec<Option<Value>>)>;
}

pub async fn query<T: EdgedbSetValue, Args: EdgedbQueryArgs + Send>(
    client: &Client,
    q: &str,
    args: Args,
) -> Result<T> {
    let val = T::query_direct(client, q, args).await?;
    Ok(val)
}

#[cfg(test)]
mod test {
    use crate::{query, value::EdgedbSetValue, EdgedbObject};

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
                            .map(EdgedbSetValue::from_edgedb_set_value)
                            .transpose()?;
                    }
                    "b" => {
                        b = fields[i]
                            .take()
                            .map(EdgedbSetValue::from_edgedb_set_value)
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

        assert_eq!(query::<i64, _>(&conn, "select 7*8", ()).await?, 56);

        assert_eq!(
            query::<ExamplImplStruct, _>(&conn, "select {a:='aaa',b:=<str>{}}", ()).await?,
            ExamplImplStruct {
                a: "aaa".to_string(),
                b: None
            }
        );

        assert_eq!(
            query::<ExamplImplStruct, _>(&conn, "select {a:='aaa',b:=<str>{'cc'}}", ()).await?,
            ExamplImplStruct {
                a: "aaa".to_string(),
                b: Some("cc".to_string())
            }
        );

        assert!(
            query::<ExamplImplStruct, _>(&conn, "select {a:='aaa',b:=<str>{'cc', 'dd'}}", ())
                .await
                .is_err()
        );

        Ok(())
    }
}
