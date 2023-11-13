use edgedb_protocol::model::Uuid;
#[doc(hidden)]
pub use itertools;

#[doc(hidden)]
pub fn query_add_indent(s: &str) -> String {
    s.replace('\n', "\n\t")
}

pub trait AsEdgedbVar {
    const EDGEDB_TYPE: &'static str;
    const IS_OPTIONAL: bool = false;

    fn full_type() -> String {
        if Self::IS_OPTIONAL {
            format!("optional {}", Self::EDGEDB_TYPE)
        } else {
            // Self::EDGEDB_TYPE.to_string()
            format!("required {}", Self::EDGEDB_TYPE)
        }
    }
}

impl AsEdgedbVar for i32 {
    const EDGEDB_TYPE: &'static str = "int32";
}
impl AsEdgedbVar for String {
    const EDGEDB_TYPE: &'static str = "str";
}
impl AsEdgedbVar for Uuid {
    const EDGEDB_TYPE: &'static str = "uuid";
}

impl<T: AsEdgedbVar> AsEdgedbVar for Option<T> {
    const EDGEDB_TYPE: &'static str = T::EDGEDB_TYPE;
    const IS_OPTIONAL: bool = true;
}

impl<T: AsEdgedbVar> ComposableQuerySelector for T {
    const RESULT_TYPE: ComposableQueryResultType = ComposableQueryResultType::Field;

    fn format_selector(_fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }

    fn format_subquery(_fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

pub enum ComposableQueryResultType {
    Field,
    Selector,
    FreeObject,
}

pub trait ComposableQuerySelector {
    const RESULT_TYPE: ComposableQueryResultType;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error>;

    fn format_subquery(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        match Self::RESULT_TYPE {
            ComposableQueryResultType::Field => {
                return Ok(());
            }
            ComposableQueryResultType::Selector => fmt.write_str(": ")?,
            ComposableQueryResultType::FreeObject => fmt.write_str(" := ")?,
        };

        Self::format_selector(fmt)
    }
}

pub trait ComposableQuery: ComposableQuerySelector {
    const ARG_NAMES: &'static [&'static str];

    type ArgTypes;
    type ReturnType;

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
