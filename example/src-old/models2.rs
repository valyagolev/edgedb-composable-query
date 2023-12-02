use edgedb_composable_query_derive::{ComposableQuery, EdgedbComposableSelector};
use edgedb_protocol::model::Uuid;

#[derive(EdgedbComposableSelector, Debug)]
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

#[derive(ComposableQuery, Debug)]
#[params(cnt: usize)]
#[select("select Inner limit cnt")]
struct AllInner(Vec<Inner>);

#[derive(ComposableQuery)]
#[params(req: String, opt: Option<String>)]
#[select("insert Inner { req := req, opt := opt }")]
struct Insert(Inner);

#[cfg(test)]
mod test {
    use edgedb_composable_query::{query, ComposableQuery, EdgedbComposableSelector};

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

    #[tokio::test]
    async fn test_me() {
        let conn = edgedb_tokio::create_client().await.unwrap();

        let res = query::<super::AllInner>(conn, (1,)).await.unwrap();

        dbg!(res);
    }
}
