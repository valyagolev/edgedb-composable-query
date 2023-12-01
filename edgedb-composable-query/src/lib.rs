/// todo: local Result type
pub use anyhow::Result;

use edgedb_tokio::Client;
pub use nonempty::{nonempty, NonEmpty};
pub mod prim;
pub mod value;
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
    use crate::query;

    #[tokio::test]
    async fn some_queries() -> anyhow::Result<()> {
        let conn = edgedb_tokio::create_client().await?;

        let v = query::<i64>(&conn, "select 7*8").await?;

        Ok(())
    }
}
