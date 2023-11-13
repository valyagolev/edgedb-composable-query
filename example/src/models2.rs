use edgedb_composable_query_derive::ComposableQuery;
use edgedb_protocol::model::Uuid;

#[derive(ComposableQuery)]
struct Inner {
    id: Uuid,
    opt: Option<String>,
    req: String,

    #[var("len(.req)")]
    strlen: i64,
}

#[derive(ComposableQuery)]
#[params(id: Uuid)]
#[select("select Inner filter .id = id limit 1")]
struct InnerById(Inner);

#[cfg(test)]
mod test {
    use edgedb_composable_query::ComposableQuery;

    #[test]
    fn show_me() {
        println!("\n\n{}", super::Inner::query());

        println!("\n\n{}", super::InnerById::query());

        // println!("{}", super::WrappedQuery::query());
    }
}
