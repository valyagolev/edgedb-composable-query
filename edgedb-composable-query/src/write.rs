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

    // #[tokio::test]
    // async fn some_queries() -> anyhow::Result<()> {
    //     let query = query!(r#"SELECT { a := "a", b := "b" }"#, ExamplImplStruct,);

    //     let expected = ExamplImplStruct {
    //         a: "a".to_owned(),
    //         b: Some("b".to_owned()),
    //     };

    //     let actual = query
    //         .first::<ExamplImplStruct>()
    //         .await
    //         .expect("query failed");

    //     assert_eq!(expected, actual);

    //     Ok(())
    // }
}
