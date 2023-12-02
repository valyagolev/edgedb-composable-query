edgedb-composable-query
========================

[![Crates.io](https://img.shields.io/crates/v/edgedb-composable-query.svg)](https://crates.io/crates/edgedb-composable-query)
[![Docs.rs](https://docs.rs/edgedb-composable-query/badge.svg)](https://docs.rs/edgedb-composable-query)
![License](https://img.shields.io/crates/l/edgedb-composable-query.svg)


Query arbitrary structs from EdgeDB. Compose queries of arbitrary complexity.

Beware: it's very much a work-in-progress. Pre-0.1. It's messy,
there're todo!()'s in the code, etc. I'm still figuring out the semantics.
If you're interested in this working for your use-cases, please
try it and file the issues at: <https://github.com/valyagolev/edgedb-composable-query/issues>.
But don't use it seriously yet; it might panic, and *will* change the API.

Two major parts of the crate:

1. A set of tools, around the [`EdgedbObject`] derivable trait, that allow you to query
arbitrary rust structs from EdgeDB, converting types automatically. See examples below: https://docs.rs/edgedb-composable-query/latest/edgedb_composable_query/

2. A set of tools, around the [`composable::EdgedbComposableQuery`] derivable trait, that allow you express
complex, composable queries through Rust structs and attributes. See docs and examples in the [composable] submodule: https://docs.rs/edgedb-composable-query/latest/edgedb_composable_query/composable/index.html