use edgedb_composable_query_derive::ComposableQuery;

mod models;
mod test2;

#[derive(ComposableQuery)]
#[params(n: i32, v: String)]
#[with(calc = "n + 2")]
// #[with(q = "insert Q {n := calc, name := n}")]
#[with(calc2 = calc)]
struct InsertQ {
    // this is for `select { q := q, calc := calc }`
    #[var(v)]
    idd: String,
    #[var("calc")]
    calc: i32,
    n: i32,
}

#[derive(ComposableQuery)]
#[params(a: i32, v: String)]
#[with(qqqq = InsertQ(n = "a + 1", v = "v"))]
// #[select("q")] // this is for `select q { v, calc }` // will not support var()
struct InsertQWrapped {
    v: String,
    idd: String,
}

fn main() {}

#[cfg(test)]
mod test {
    use edgedb_composable_query::ComposableQuery;

    #[test]
    fn show_me() {
        println!("{}", super::InsertQ::query());

        println!("{}", super::InsertQWrapped::query());
    }
}

// this one should be like
/*
with
    a := <i32>$0,
    v := <str>$1,
    q := (with ...... ),
select
    q {
        v,
        calc,
    }

so....
    1. query should be a function(&[&str]) -> String
    2. query will handle its input types by itself

*/
