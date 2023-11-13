use edgedb_composable_query_derive::ComposableQuery;

#[derive(ComposableQuery)]
#[select("select Inner limit 1")]
struct Inner {
    opt: Option<String>,
    req: String,

    #[var("len(.req)")]
    strlen: i64,
}

#[derive(ComposableQuery)]
#[select("select Outer limit 1")]
struct Outer {
    inner: Inner,
    other_field: String,
}

#[cfg(test)]
mod test {
    use edgedb_composable_query::ComposableQuery;

    #[test]
    fn show_me() {
        println!("\n\n{}", super::Inner::query());

        println!("\n\n{}", super::Outer::query());

        // println!("{}", super::WrappedQuery::query());
    }
}
