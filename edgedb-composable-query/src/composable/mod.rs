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
//! use edgedb_composable_query::{query, EdgedbObject, Ref};
//! use edgedb_composable_query::composable::{EdgedbComposableQuery, EdgedbComposableSelector};
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
//! struct InnerSelector {
//!    req: String,
//!    opt: Option<String>,
//! }
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
//! struct OuterSelector {
//!     inner: Option<InnerSelector>,
//!
//!     some_field: Option<String>,
//!     other_field: String,
//! }
//!
//! #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
//! struct OuterSelectorWithRef {
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

pub use edgedb_composable_query_derive::EdgedbComposableSelector;

pub enum ComposableQueryResultKind {
    Field,
    Selector,
    FreeObject,
}

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

impl<T: EdgedbComposableSelector + EdgedbObject> EdgedbComposableSelector for Ref<T> {
    const RESULT_TYPE: ComposableQueryResultKind = ComposableQueryResultKind::Selector;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        fmt.write_str("\tid,\n")?;

        T::format_selector(fmt)?;

        Ok(())
    }
}

pub trait EdgedbComposableQuery: EdgedbComposableSelector {
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

#[cfg(test)]
mod test {
    use crate::composable::EdgedbComposableSelector;
    use crate::{EdgedbObject, Ref};

    #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
    struct InnerSelector {
        req: String,
        opt: Option<String>,
    }

    #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
    struct OuterSelector {
        inner: Option<InnerSelector>,

        some_field: Option<String>,
        other_field: String,
    }

    #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
    struct OuterSelectorWithRef {
        inner: Option<Ref<InnerSelector>>,

        some_field: Option<String>,
        other_field: String,
    }

    #[test]
    fn selector_tests() {
        let mut buf = String::new();
        InnerSelector::format_selector(&mut buf).unwrap();
        insta::assert_snapshot!(buf);

        let mut buf = String::new();
        OuterSelector::format_selector(&mut buf).unwrap();
        insta::assert_snapshot!(buf);

        let mut buf = String::new();
        OuterSelectorWithRef::format_selector(&mut buf).unwrap();
        insta::assert_snapshot!(buf);
    }
}
