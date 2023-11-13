use edgedb_composable_query_derive::ComposableQuery;

#[derive(ComposableQuery)]
#[select("select Inner limit 1")]
struct Inner {
    opt: Option<String>,
    req: String,
}

#[cfg(test)]
mod test {
    use edgedb_composable_query::ComposableQuery;

    #[test]
    fn show_me() {
        println!("{}", super::Inner::query());

        // println!("{}", super::WrappedQuery::query());
    }
}
