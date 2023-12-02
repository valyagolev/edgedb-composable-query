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
//! Here're some of the ways to use `EdgedbComposableQuery`:

use crate::{EdgedbQueryArgs, EdgedbSetValue};

pub enum ComposableQueryResultKind {
    Field,
    Selector,
    FreeObject,
}

pub trait EdgedbComposableSelector {
    const RESULT_TYPE: ComposableQueryResultKind;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error>;

    fn format_subquery(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        match Self::RESULT_TYPE {
            ComposableQueryResultKind::Field => {
                return Ok(());
            }
            ComposableQueryResultKind::Selector => fmt.write_str(": ")?,
            ComposableQueryResultKind::FreeObject => fmt.write_str(" := ")?,
        };

        Self::format_selector(fmt)
    }
}

// impl<T: EdgedbComposableSelector> EdgedbComposableSelector for Vec<T> {
//     const RESULT_TYPE: ComposableQueryResultKind = T::RESULT_TYPE;

//     fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
//         T::format_selector(fmt)
//     }
// }

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
    use super::EdgedbComposableSelector;
    use crate::EdgedbObject;
    use crate::EdgedbSetValue;

    #[derive(Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector)]
    struct InnerSelector {
        req: String,
        opt: Option<String>,
    }

    #[test]
    fn sync_tests() {
        dbg!()
    }
}
