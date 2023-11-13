edgedb-composable-query
=======================


```rust

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


```


Will result in:

```edgeql

with
        n := <int32>$0,
        v := <str>$1,
        calc := (n + 2),
select {
        n := (n),
}


with
        n := <int32>$0,
        inner_res := (with
                n := <int32>(n + 5),
                v := <str>('whatever'),
                calc := (n + 2),
        select {
                n := (n),
        }),
select {
        q := (inner_res.n),
}
```