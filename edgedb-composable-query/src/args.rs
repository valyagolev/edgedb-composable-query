use crate::EdgedbValue;
use crate::{EdgedbObject, Result};
use edgedb_protocol::query_arg::QueryArgs;
use edgedb_protocol::value::Value;

pub trait EdgedbQueryArgs {
    type EdgedbArgsType: QueryArgs;

    fn as_query_args(self) -> Result<Self::EdgedbArgsType>;
}

impl EdgedbQueryArgs for () {
    type EdgedbArgsType = ();

    fn as_query_args(self) -> Result<Self::EdgedbArgsType> {
        Ok(self)
    }
}

macro_rules! ignore_first {
    ($a:ident, $b:ident) => {
        $b
    };
}

macro_rules! impl_tuple {
    ( $count:expr, ($($name:ident,)+), ($($small_name:ident,)+) ) => (

        impl<$($name:EdgedbValue),+> EdgedbQueryArgs for ($($name,)+) {
            type EdgedbArgsType = ($(ignore_first!($name, Value),)+);

            fn as_query_args(self) -> Result<Self::EdgedbArgsType> {
                let ($($small_name,)+) = self;

                Ok(($($small_name.to_edgedb_value()?,)+))
            }
        }

    )
}

impl_tuple! {1, (T0,), (t0,)}
impl_tuple! {2, (T0, T1,), (t0, t1,)}
impl_tuple! {3, (T0, T1, T2,), (t0, t1, t2,)}
impl_tuple! {4, (T0, T1, T2, T3,), (t0, t1, t2, t3,)}
impl_tuple! {5, (T0, T1, T2, T3, T4,), (t0, t1, t2, t3, t4,)}
impl_tuple! {6, (T0, T1, T2, T3, T4, T5,), (t0, t1, t2, t3, t4, t5,)}

#[cfg(test)]
mod test {
    use crate::{
        query,
        value::{EdgedbSetValue, EdgedbValue},
        EdgedbObject,
    };

    #[derive(Debug, PartialEq)]
    struct ExamplImplStruct {
        a: String,
        b: Option<String>,
    }

    impl EdgedbObject for ExamplImplStruct {
        fn from_edgedb_object(
            shape: edgedb_protocol::codec::ObjectShape,
            mut fields: Vec<Option<edgedb_protocol::value::Value>>,
        ) -> anyhow::Result<Self> {
            let mut a = None;
            let mut b = None;

            for (i, s) in shape.elements.iter().enumerate() {
                match s.name.as_str() {
                    "a" => {
                        a = fields[i]
                            .take()
                            .map(|v| EdgedbSetValue::from_edgedb_set_value(v))
                            .transpose()?;
                    }
                    "b" => {
                        b = fields[i]
                            .take()
                            .map(|v| EdgedbSetValue::from_edgedb_set_value(v))
                            .transpose()?;
                    }
                    _ => {}
                }
            }

            Ok(Self {
                a: EdgedbSetValue::interpret_possibly_missing_required_value(a)?,
                b: EdgedbSetValue::interpret_possibly_missing_required_value(b)?,
            })
        }

        fn to_edgedb_object(
            &self,
        ) -> anyhow::Result<(
            edgedb_protocol::codec::ObjectShape,
            Vec<Option<edgedb_protocol::value::Value>>,
        )> {
            todo!()
        }
    }

    #[tokio::test]
    async fn some_queries() -> anyhow::Result<()> {
        let conn = edgedb_tokio::create_client().await?;

        // assert_eq!(query::<i64, _>(&conn, "select 7*8", ()).await?, 56);

        assert_eq!(
            query::<ExamplImplStruct, _>(
                &conn,
                "select {a:=<str>$0,b:=<str>{}}",
                ("hi".to_owned(),)
            )
            .await?,
            ExamplImplStruct {
                a: "hi".to_string(),
                b: None
            }
        );

        // assert_eq!(
        //     query::<ExamplImplStruct, _>(&conn, "select {a:='aaa',b:=<str>{'cc'}}", ()).await?,
        //     ExamplImplStruct {
        //         a: "aaa".to_string(),
        //         b: Some("cc".to_string())
        //     }
        // );

        // assert!(
        //     query::<ExamplImplStruct, _>(&conn, "select {a:='aaa',b:=<str>{'cc', 'dd'}}", ())
        //         .await
        //         .is_err()
        // );

        Ok(())
    }
}
