use edgedb_composable_query_derive::ComposableQuery;

#[derive(ComposableQuery)]
#[params(n: i32, v: String)]
#[with(calc = "n + 2")]
struct SomeQuery {
    n: i32,
}

#[derive(ComposableQuery)]
#[params(n: i32)]
#[with(inner_res = SomeQuery(n = "n + 5", v = "'whatever'"))]
struct WrappedQuery {
    #[var("inner_res.n")]
    q: String,
}

#[cfg(test)]
mod test {
    use edgedb_composable_query::ComposableQuery;

    #[test]
    fn show_me() {
        println!("{}", super::SomeQuery::query());

        println!("{}", super::WrappedQuery::query());
    }
}
