use edgedb_composable_query_derive::{ComposableQuery, ComposableQuerySelector};
use edgedb_protocol::model::Uuid;

#[derive(ComposableQuerySelector)]
struct Inner {
    id: Uuid,
    opt: Option<String>,
    req: String,

    #[var("len(.req)")]
    strlen: i64,
}

#[derive(ComposableQuery)]
#[params(id: Uuid)]
#[select("select Inner filter .id = id")]
struct InnerById(Inner);

#[derive(ComposableQuery)]
#[params(cnt: usize)]
#[select("select Inner limit cnt")]
struct AllInner(Vec<Inner>);

#[derive(ComposableQuery)]
#[params(req: String, opt: Option<String>)]
#[select("insert Inner { req := req, opt := opt }")]
struct Insert(Inner);

#[cfg(test)]
mod test {
    use edgedb_composable_query::{ComposableQuery, ComposableQuerySelector};

    #[test]
    fn show_me() {
        let mut buf = String::new();
        super::Inner::format_selector(&mut buf).unwrap();

        println!("\n\n{}", buf);

        println!("\n\n{}", super::InnerById::query());

        println!("\n\n{}", super::Insert::query());

        println!("\n\n{}", super::AllInner::query());

        // println!("{}", super::WrappedQuery::query());
    }
}
