//! # edgedb_composable_query::composable
//!
//! Beware: it's very much a work-in-progress. Pre-0.1. There're todo!()'s in the code, etc.
//! I'm still figuring out the semantics.
//! If you're interested in this working for your use-cases, please
//! try it and file the issues at: <https://github.com/valyagolev/edgedb-composable-query/issues>.
//! But don't use it seriously yet; it might panic, and *will* change the API.
//!
//! # EdgedbComposableQuery Examples
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
//! Here're some of the ways to use this module:
//!
//! ```
//! # tokio_test::block_on(async {
//! use edgedb_protocol::model::Uuid;
//! use edgedb_composable_query::{query, EdgedbObject, Ref};
//! use edgedb_composable_query::composable::{EdgedbComposableQuery, EdgedbComposableSelector, run_query};
//!
//! let conn = edgedb_tokio::create_client().await?;
//!
//! // You can define specific queries directly:
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbObject,
//!          EdgedbComposableSelector, EdgedbComposableQuery)]
//! #[select("select Inner limit 1")]
//! struct OneInnerQuery {
//!    req: String,
//!    opt: Option<String>,
//! }
//!
//! assert!(
//!     run_query::<OneInnerQuery>(&conn, ()).await.is_ok()
//! );
//!
//! // If you want to compose them, typically it's better to extract the selector:
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
//! struct InnerSelector {
//!   req: String,
//!   opt: Option<String>,
//! }
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbComposableQuery)]
//! #[select("select Inner limit 1")]
//! struct OneInnerBySelector(InnerSelector);
//!
//! assert!(
//!     run_query::<OneInnerBySelector>(&conn, ()).await.is_ok()
//! );
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbComposableQuery)]
//! #[params(id: Uuid)]
//! #[select("select Inner filter .id = id")]
//! struct OneInnerBySelectorById(Option<InnerSelector>);
//!
//! assert!(
//!     run_query::<OneInnerBySelectorById>(&conn, (
//!         Uuid::parse_str("9be70fb0-8240-11ee-9175-cff95b46d325").unwrap(),
//!     )).await.unwrap().is_some()
//! );
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbComposableQuery)]
//! #[select("select Inner limit 10")]
//! struct ManyInnersBySelector(Vec<InnerSelector>);
//!
//! // Queries can have parameters:
//! // (And remember to use Ref<T>)
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbObject,
//!          EdgedbComposableSelector, EdgedbComposableQuery)]
//! #[params(id: Uuid)]
//! #[select("select Outer filter .id = id limit 1")]
//! struct OuterByIdQuery {
//!     inner: Option<Ref<InnerSelector>>,
//!
//!     some_field: Option<String>,
//!     other_field: String,
//! }
//!
//! # anyhow::Ok(())
//! # }).unwrap();
//! ```

use crate::{EdgedbObject, EdgedbPrim, EdgedbQueryArgs, EdgedbSetValue, Ref};

pub use edgedb_composable_query_derive::{EdgedbComposableQuery, EdgedbComposableSelector};
use edgedb_tokio::Client;
use nonempty::NonEmpty;

use crate::Result;

pub enum ComposableQueryResultKind {
    Field,
    Selector,
    FreeObject,
}

/// Derivable trait. Must have named fields, each is either another selector, or a primitive, or a `Vec/Option/NonEmpty` of those.
pub trait EdgedbComposableSelector {
    const RESULT_TYPE: ComposableQueryResultKind;

    /// should't add `{` and `}` around the selector
    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error>;

    fn format_subquery(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        match Self::RESULT_TYPE {
            ComposableQueryResultKind::Field => {
                return Ok(());
            }
            ComposableQueryResultKind::Selector => fmt.write_str(": {\n")?,
            ComposableQueryResultKind::FreeObject => fmt.write_str(" := {\n")?,
        };

        Self::format_selector(fmt)?;

        fmt.write_str("\n}")
    }
}

impl<T: EdgedbPrim> EdgedbComposableSelector for T {
    const RESULT_TYPE: ComposableQueryResultKind = ComposableQueryResultKind::Field;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }

    fn format_subquery(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl<T: EdgedbComposableSelector> EdgedbComposableSelector for Vec<T> {
    const RESULT_TYPE: ComposableQueryResultKind = T::RESULT_TYPE;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        T::format_selector(fmt)
    }
}

impl<T: EdgedbComposableSelector> EdgedbComposableSelector for Option<T> {
    const RESULT_TYPE: ComposableQueryResultKind = T::RESULT_TYPE;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        T::format_selector(fmt)
    }
}

impl<T: EdgedbComposableSelector> EdgedbComposableSelector for NonEmpty<T> {
    const RESULT_TYPE: ComposableQueryResultKind = T::RESULT_TYPE;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        T::format_selector(fmt)
    }
}

impl<T: EdgedbComposableSelector + EdgedbObject> EdgedbComposableSelector for Ref<T> {
    const RESULT_TYPE: ComposableQueryResultKind = ComposableQueryResultKind::Selector;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        fmt.write_str("\tid,\n")?;

        T::format_selector(fmt)?;

        Ok(())
    }
}

/// Derivable trait. Can have parameters. Either an object with named fields, or can be a wrapper around a selector, or `Option<selector>`, or `Vec<selector>`, or `NonEmpty<selector>``.
pub trait EdgedbComposableQuery {
    const ARG_NAMES: &'static [&'static str];

    type ArgTypes: EdgedbQueryArgs;
    type ReturnType: EdgedbSetValue;

    fn format_query(
        fmt: &mut impl std::fmt::Write,
        args: &::std::collections::HashMap<&str, String>,
    ) -> Result<(), std::fmt::Error>;

    fn query() -> String {
        let mut buf = String::new();
        // let args = (0..Self::ARG_NAMES.len())
        //     .map(|i| (format!("${}", i)))
        //     .collect();

        let args = Self::ARG_NAMES
            .iter()
            .enumerate()
            .map(|(i, n)| (*n, format!("${i}")))
            .collect();

        Self::format_query(&mut buf, &args).unwrap();
        buf
    }
}

/// use this to run an [`EdgedbComposableQuery`]
pub async fn run_query<T: EdgedbComposableQuery>(
    client: &Client,
    args: T::ArgTypes,
) -> Result<T::ReturnType>
where
    <T as EdgedbComposableQuery>::ArgTypes: Send,
{
    let query_s = T::query();

    crate::query(client, &query_s, args).await
}

#[cfg(test)]
mod test {
    use edgedb_protocol::model::Uuid;

    use crate::composable::EdgedbComposableQuery;
    use crate::composable::EdgedbComposableSelector;
    use crate::{EdgedbObject, Ref};

    #[derive(
        Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector, EdgedbComposableQuery,
    )]
    #[select("select Inner limit 1")]
    struct InnerQuery {
        req: String,
        opt: Option<String>,
    }

    #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
    struct InnerSelector {
        req: String,
        opt: Option<String>,
    }

    #[derive(Debug, PartialEq, Eq, EdgedbComposableQuery)]
    #[select("select Inner limit 1")]
    struct OneInnerBySelector(InnerSelector);

    #[derive(Debug, PartialEq, Eq, EdgedbComposableQuery)]
    #[params(id: Uuid)]
    #[select("select Inner filter .id = id")]
    struct OneInnerBySelectorById(InnerSelector);

    #[derive(Debug, PartialEq, Eq, EdgedbComposableQuery)]
    #[select("select Inner limit 10")]
    struct ManyInnersBySelector(Vec<InnerSelector>);

    #[derive(
        Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector, EdgedbComposableQuery,
    )]
    #[select("select Outer limit 1")]
    struct OuterQuery {
        inner: Option<InnerSelector>,

        some_field: Option<String>,
        other_field: String,
    }

    #[derive(
        Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector, EdgedbComposableQuery,
    )]
    #[select("select Outer limit 1")]
    struct OuterQueryWithRef {
        inner: Option<Ref<InnerSelector>>,

        some_field: Option<String>,
        other_field: String,
    }

    #[test]
    fn selector_tests() {
        // let mut buf = String::new();
        // InnerQuery::format_selector(&mut buf).unwrap();
        // insta::assert_snapshot!(buf);

        // let mut buf = String::new();
        // OuterQuery::format_selector(&mut buf).unwrap();
        // insta::assert_snapshot!(buf);

        // let mut buf = String::new();
        // OuterQueryWithRef::format_selector(&mut buf).unwrap();
        // insta::assert_snapshot!(buf);

        let mut buf = String::new();
        InnerSelector::format_selector(&mut buf).unwrap();
        insta::assert_snapshot!(buf);

        let mut buf = String::new();
        Option::<InnerSelector>::format_selector(&mut buf).unwrap();
        insta::assert_snapshot!(buf);
    }

    #[test]
    fn query_tests() {
        insta::assert_snapshot!(InnerQuery::query());
        insta::assert_snapshot!(OuterQuery::query());
        insta::assert_snapshot!(OuterQueryWithRef::query());

        insta::assert_snapshot!(OneInnerBySelector::query());
        insta::assert_snapshot!(OneInnerBySelectorById::query());
        insta::assert_snapshot!(ManyInnersBySelector::query());
    }
}
