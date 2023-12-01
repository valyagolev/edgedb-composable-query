use edgedb_protocol::value::Value;

use crate::value::{EdgedbSetValue, EdgedbValue};

impl EdgedbValue for () {
    type NativeArgType = ();

    fn from_edgedb_value(value: Value) -> anyhow::Result<Self> {
        if let Value::Nothing = value {
            Ok(())
        } else {
            Err(anyhow::anyhow!("expected nothing"))
        }
    }

    fn to_edgedb_value(self) -> anyhow::Result<Value> {
        Ok(Value::Nothing)
    }
}

macro_rules! impl_tuple {
    ( $count:expr, ($($name:ident,)+), ($($small_name:ident,)+) ) => (

        impl<$($name:EdgedbValue),+> EdgedbValue for ($($name,)+) {
            // const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
            //     edgedb_protocol::server_message::Cardinality::One;

            type NativeArgType = ($(<$name as EdgedbValue>::NativeArgType),+);

            fn from_edgedb_value(value: edgedb_protocol::value::Value) -> anyhow::Result<Self> {
                if let Value::Tuple(mut v) = value {
                    if v.len() != $count {
                        return Err(anyhow::anyhow!(
                            "expected tuple of length {}, got {}",
                            $count,
                            v.len()
                        ));
                    }

                    Ok(($($name::from_edgedb_set_value(v.pop().unwrap())?,)+))
                } else {
                    Err(anyhow::anyhow!("expected tuple"))
                }
            }

            fn to_edgedb_value(self) -> anyhow::Result<edgedb_protocol::value::Value> {
                let ($($small_name,)+) = self;
                Ok(Value::Tuple(vec![$($small_name.to_edgedb_set_value()?),+]))
            }

            // fn interpret_possibly_missing_required_value(val: Option<Self>) -> anyhow::Result<Self> {
            //     match val {
            //         Some(val) => Ok(val),
            //         None => Err(anyhow::anyhow!("expected single value")),
            //     }
            // }

            // async fn query_direct(client: &edgedb_tokio::Client, q: &str) -> anyhow::Result<Self> {
            //     let val = client.query_required_single::<Value, _>(q, &()).await?;
            //     let val = Self::from_edgedb_set_value(val)?;
            //     Ok(val)
            // }
        }

    )
}

// macro_rules! impl_tuple {
//     ( $count:expr, ($($name:ident,)+), ($($small_name:ident,)+) ) => (

//         impl<$($name:EdgedbSetValue),+> EdgedbSetValue for ($($name,)+) {
//             const EXPECTED_CARDINALITY: edgedb_protocol::server_message::Cardinality =
//                 edgedb_protocol::server_message::Cardinality::One;

//             fn from_edgedb_set_value(value: edgedb_protocol::value::Value) -> anyhow::Result<Self> {
//                 if let Value::Tuple(mut v) = value {
//                     if v.len() != $count {
//                         return Err(anyhow::anyhow!(
//                             "expected tuple of length {}, got {}",
//                             $count,
//                             v.len()
//                         ));
//                     }

//                     Ok(($($name::from_edgedb_set_value(v.pop().unwrap())?,)+))
//                 } else {
//                     Err(anyhow::anyhow!("expected tuple"))
//                 }
//             }

//             fn to_edgedb_set_value(self) -> anyhow::Result<edgedb_protocol::value::Value> {
//                 let ($($small_name,)+) = self;
//                 Ok(Value::Tuple(vec![$($small_name.to_edgedb_set_value()?),+]))
//             }

//             fn interpret_possibly_missing_required_value(val: Option<Self>) -> anyhow::Result<Self> {
//                 match val {
//                     Some(val) => Ok(val),
//                     None => Err(anyhow::anyhow!("expected single value")),
//                 }
//             }

//             async fn query_direct<Args: EdgedbValue>(client: &edgedb_tokio::Client, q: &str, args: Args) -> anyhow::Result<Self> {
//                 let val = client.query_required_single::<Value, _>(q, &()).await?;
//                 let val = Self::from_edgedb_set_value(val)?;
//                 Ok(val)
//             }
//         }

//     )
// }

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
        refs::Ref,
        value::{EdgedbSetValue, EdgedbValue},
        EdgedbObject,
    };

    #[derive(Debug, PartialEq)]
    struct Inner {
        req: String,
        opt: Option<String>,
    }

    impl EdgedbObject for Inner {
        fn from_edgedb_object(
            shape: edgedb_protocol::codec::ObjectShape,
            mut fields: Vec<Option<edgedb_protocol::value::Value>>,
        ) -> anyhow::Result<Self> {
            let mut req = None;
            let mut opt = None;

            for (i, s) in shape.elements.iter().enumerate() {
                match s.name.as_str() {
                    "req" => {
                        req = fields[i]
                            .take()
                            .map(|v| EdgedbSetValue::from_edgedb_set_value(v))
                            .transpose()?;
                    }
                    "opt" => {
                        opt = fields[i]
                            .take()
                            .map(|v| EdgedbSetValue::from_edgedb_set_value(v))
                            .transpose()?;
                    }
                    _ => {}
                }
            }

            Ok(Self {
                req: EdgedbSetValue::interpret_possibly_missing_required_value(req)?,
                opt: EdgedbSetValue::interpret_possibly_missing_required_value(opt)?,
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

        dbg!(
            query::<(Ref<Inner>, Ref<Inner>), ()>(
                &conn,
                "
            with a := (select Inner {id, opt, req} limit 1),
            select (a, a)
            ",
                ()
            )
            .await?
        );

        Ok(())
    }
}
