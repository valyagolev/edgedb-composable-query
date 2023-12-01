use edgedb_composable_query_derive::ComposableQuery;

#[derive(ComposableQuery)]
#[params(n: i32, v: String)]
#[with(calc = "n + 2")]
#[with(q = "insert Q {n := calc, name := n}")]
#[with(calc2 = calc)]
struct InsertQ {
    // this is for `select { q := q, calc := calc }`
    #[var(q)]
    id: String,
    #[var("calc")]
    calc: i32,
    by_name: i32,
}

#[derive(ComposableQuery)]
#[params(a: i32, v: String)]
#[with(q = InsertQ(n = "a + 1", v = "v"))]
#[select("q")] // this is for `select q { v, calc }` // will not support var()
struct InsertQWrapped {
    v: String,
    id: String,
}

fn main() {}
