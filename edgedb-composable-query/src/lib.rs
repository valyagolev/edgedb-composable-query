#[doc(hidden)]
pub use itertools;

#[doc(hidden)]
pub fn query_add_indent(s: &str) -> String {
    s.replace('\n', "\n\t")
}

pub trait AsEdgedbVar {
    const EDGEDB_TYPE: &'static str;
}

impl AsEdgedbVar for i32 {
    const EDGEDB_TYPE: &'static str = "int32";
}
impl AsEdgedbVar for String {
    const EDGEDB_TYPE: &'static str = "str";
}

pub trait ComposableQuery {
    const ARG_NAMES: &'static [&'static str];

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
