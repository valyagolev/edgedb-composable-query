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

// #[derive(ComposableQuery)]
// #[select("select Outer limit 1")]
// struct Outer {
//     id: Uuid,
//     inner: Inner,
//     other_field: String,
// }

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
