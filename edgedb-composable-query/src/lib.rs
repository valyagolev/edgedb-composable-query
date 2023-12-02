//! # edgedb-composable-query
//!
//! Query arbitrary structs from EdgeDB. Compose queries of arbitrary complexity.
//!
//! Beware: it's very much a work-in-progress. Pre-0.1. It's messy,
//! there're todo!()'s in the code, etc. I'm still figuring out the semantics.
//! If you're interested in this working for your use-cases, please
//! try it and file the issues at: <https://github.com/valyagolev/edgedb-composable-query/issues>.
//! But don't use it seriously yet; it might panic, and *will* change the API.
//!
//! Two major parts of the crate.
//!
//! 1. A set of tools, around the [`EdgedbObject`] derivable trait, that allow you to query
//! arbitrary rust structs from EdgeDB, converting types automatically. See examples below.
//! 2. A set of tools, around the [`composable::EdgedbComposableQuery`] derivable trait, that allow you express
//! complex, composable queries through Rust structs and attributes. See docs and examples in the [composable] submodule.
//!
//! # EdgedbObject Examples
//!
//! If you have this schema:
//!
//! ```edgedb
//! module default {
//! type Inner {
//!     required req: str;
//!     opt: str;
//! }
//! type Outer {
//!     inner: Inner;
//!
//!     some_field: str;
//!     required other_field: str;
//! }
//! ```
//!
//! Here're some of the ways to use `EdgedbObject`:
//!
//! ```
//! # tokio_test::block_on(async {
//! use edgedb_composable_query::{query, EdgedbObject, EdgedbSetValue, Ref};
//!
//! #[derive(Debug, PartialEq, EdgedbObject)]
//! struct AdHocStruct {
//!     a: String,
//!     b: Option<String>,
//! }
//!
//! #[derive(Debug, PartialEq, EdgedbObject)]
//! struct Inner {
//!     req: String,
//!     opt: Option<String>,
//! }
//!
//!
//! // typically you want to use Ref<T> to refer to a type
//! // Ref<T> is basically UUID and an Option<T>
//!
//! #[derive(Debug, PartialEq, EdgedbObject)]
//! struct Outer {
//!     inner: Option<Ref<Inner>>,
//!     some_field: Option<String>,
//!     other_field: String,
//! }
//!
//! let conn = edgedb_tokio::create_client().await?;
//!
//! assert_eq!(query::<i64, _>(&conn, "select 7*8", ()).await?, 56);
//!
//! // use primitive params
//! assert_eq!(
//!     query::<Vec<i64>, _>(&conn, "select {1 * <int32>$0, 2 * <int32>$0}", (22,)).await?,
//!     vec![22, 44]
//! );
//!
//! // ad-hoc objects:
//! assert_eq!(
//!     query::<AdHocStruct, _>(&conn, "select { a := 'aaa', b := <str>{} }", ()).await?,
//!     AdHocStruct {
//!         a: "aaa".to_string(),
//!         b: None
//!     }
//! );
//!
//! assert_eq!(
//!     query::<AdHocStruct, _>(&conn, "select { a:= 'aaa', b:=<str>{'cc'} }", ()).await?,
//!     AdHocStruct {
//!         a: "aaa".to_string(),
//!         b: Some("cc".to_string())
//!     }
//! );
//!
//! // cardinality mismatch:
//! assert!(
//!     query::<AdHocStruct, _>(&conn, "select {a := 'aaa',b := <str>{'cc', 'dd'}}", ())
//!         .await
//!         .is_err()
//! );
//!
//! // look up some objects
//! assert!(
//!     dbg!(
//!         query::<Vec<Inner>, _>(&conn, "select Inner {req, opt}", ())
//!             .await?
//!     ).len()
//!     > 0
//! );
//!
//! // use ref if you need ids
//! assert!(
//!     dbg!(
//!         query::<Vec<Ref<Inner>>, _>(&conn, "select Inner {id, req, opt}", ())
//!             .await?
//!     ).len()
//!     > 0
//! );
//!
//!
//! // ref doesn't need the rest of the object
//! assert!(
//!     dbg!(
//!         query::<Vec<Ref<Inner>>, _>(&conn, "select Inner {id}", ())
//!             .await?
//!     ).len()
//!     > 0
//! );
//!
//! // cardinality mismatch:
//! assert!(
//!    query::<Ref<Inner>, _>(&conn, "select Inner {id}", ())
//!       .await
//!      .is_err()
//! );
//!
//! // you can query things with refs in them:
//!
//! let vs = query::<Vec<Outer>, _>(&conn, "select Outer {inner, some_field, other_field}", ())
//!          .await?;
//!
//! assert!(vs.len() > 0);
//!
//!
//! // refs picked up only ids here
//! assert!(
//!     vs.iter()
//!         .filter_map(|v| v.inner.as_ref())
//!         .all(|inner_ref| inner_ref.known_value.is_none())
//! );
//!
//!
//!
//! // if you want the whole object with Ref, don't forget to provide 'id' selector
//! let vs = query::<Vec<Outer>, _>(&conn, "
//! select Outer {
//!     inner: { id, req, opt },
//!     some_field,
//!     other_field
//! }
//! ", ())
//!          .await?;
//!
//! assert!(vs.len() > 0);
//!
//! // refs picked up the whole objects
//! assert!(
//!     vs.iter()
//!         .filter_map(|v| v.inner.as_ref())
//!         .all(|inner_ref| inner_ref.known_value.is_some())
//! );
//!
//! # anyhow::Ok(())
//! # }).unwrap();
//! ```

/// currently anyhow::Result. TODO: make crate's own Result type
pub use anyhow::Result;

// pub use edg

use edgedb_tokio::Client;

mod args;
pub use args::EdgedbQueryArgs;
mod prim;
pub use prim::{EdgedbJson, EdgedbPrim};
pub mod composable;
mod refs;
mod tuples;
mod value;

pub use edgedb_composable_query_derive::EdgedbObject;
pub use refs::Ref;

use edgedb_protocol::{codec::ObjectShape, value::Value};

pub use nonempty::{nonempty, NonEmpty};
pub use value::{EdgedbSetValue, EdgedbValue};

pub trait EdgedbObject: Sized {
    fn from_edgedb_object(shape: ObjectShape, fields: Vec<Option<Value>>) -> Result<Self>;
    // fn to_edgedb_object(&self) -> Result<(ObjectShape, Vec<Option<Value>>)>;
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
    use crate::{query, EdgedbObject, EdgedbSetValue};

    #[derive(Debug, PartialEq, EdgedbObject)]
    struct ExamplImplStruct {
        a: String,
        b: Option<String>,
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
