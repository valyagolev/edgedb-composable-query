use edgedb_protocol::{
    model::Uuid,
    query_arg::{QueryArg, QueryArgs},
    value::Value,
    QueryResult,
};
#[doc(hidden)]
pub use itertools;

#[doc(hidden)]
pub fn query_add_indent(s: &str) -> String {
    s.replace('\n', "\n\t")
}

pub trait AsPrimitiveEdgedbVar {
    const EDGEDB_TYPE_NAME: &'static str;
    // const IS_OPTIONAL: bool = false;

    fn as_query_arg(&self) -> Value;
    fn from_query_result(t: Value) -> Self;
}

pub trait EdgedbArgs {
    const EDGEDB_TYPE_NAME: Option<&'static str>;
    const IS_OPTIONAL: bool;

    fn type_cast() -> String {
        let Some(name) = Self::EDGEDB_TYPE_NAME else {
            return "".to_string();
        };

        if Self::IS_OPTIONAL {
            format!("<optional {}>", name)
        } else {
            format!("<required {}>", name)
        }
    }

    fn as_query_arg(&self) -> Value;
    fn from_query_result(t: Value) -> Self;
}

impl AsPrimitiveEdgedbVar for i32 {
    const EDGEDB_TYPE_NAME: &'static str = "int32";

    fn as_query_arg(&self) -> Value {
        (*self).into()
    }

    fn from_query_result(t: Value) -> Self {
        match t {
            Value::Int16(v) => v as i32,
            Value::Int32(v) => v,
            Value::Int64(v) => v as i32,
            _ => panic!("invalid type"),
        }
    }
}

impl AsPrimitiveEdgedbVar for usize {
    const EDGEDB_TYPE_NAME: &'static str = "int64";

    fn as_query_arg(&self) -> Value {
        (*self as i64).into()
    }

    fn from_query_result(t: Value) -> Self {
        match t {
            Value::Int16(v) => v as usize,
            Value::Int32(v) => v as usize,
            Value::Int64(v) => v as usize,
            _ => panic!("invalid type"),
        }
    }
}
impl AsPrimitiveEdgedbVar for String {
    const EDGEDB_TYPE_NAME: &'static str = "str";

    fn as_query_arg(&self) -> Value {
        self.clone().into()
    }

    fn from_query_result(t: Value) -> Self {
        match t {
            Value::Str(v) => v,
            _ => panic!("invalid type"),
        }
    }
}
impl AsPrimitiveEdgedbVar for Uuid {
    const EDGEDB_TYPE_NAME: &'static str = "uuid";

    fn as_query_arg(&self) -> Value {
        Value::Uuid(self.clone())
    }

    fn from_query_result(t: Value) -> Self {
        match t {
            Value::Uuid(v) => v,
            _ => panic!("invalid type"),
        }
    }
}

impl<T: AsPrimitiveEdgedbVar> EdgedbArgs for T {
    const EDGEDB_TYPE_NAME: Option<&'static str> =
        Some(<T as AsPrimitiveEdgedbVar>::EDGEDB_TYPE_NAME);
    const IS_OPTIONAL: bool = false;

    fn as_query_arg(&self) -> Value {
        <T as AsPrimitiveEdgedbVar>::as_query_arg(self)
    }

    fn from_query_result(t: Value) -> Self {
        <T as AsPrimitiveEdgedbVar>::from_query_result(t)
    }
}

impl<T: AsPrimitiveEdgedbVar> EdgedbArgs for Option<T> {
    const EDGEDB_TYPE_NAME: Option<&'static str> =
        Some(<T as AsPrimitiveEdgedbVar>::EDGEDB_TYPE_NAME);
    const IS_OPTIONAL: bool = true;

    fn as_query_arg(&self) -> Value {
        match self {
            Some(t) => t.as_query_arg(),
            None => Value::Nothing,
        }
    }

    fn from_query_result(t: Value) -> Self {
        if t == Value::Nothing {
            None
        } else {
            Some(T::from_query_result(t))
        }
    }
}

impl<T: AsPrimitiveEdgedbVar> EdgedbComposableSelector for T {
    const RESULT_TYPE: ComposableQueryResultKind = ComposableQueryResultKind::Field;

    fn format_selector(_fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }

    fn format_subquery(_fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl<T: EdgedbComposableSelector> EdgedbComposableSelector for Option<T> {
    const RESULT_TYPE: ComposableQueryResultKind = ComposableQueryResultKind::Field;

    fn format_selector(_fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }

    fn format_subquery(_fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

pub enum ComposableQueryResultKind {
    Field,
    Selector,
    FreeObject,
}

pub trait EdgedbComposableSelector {
    const RESULT_TYPE: ComposableQueryResultKind;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error>;

    fn format_subquery(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        match Self::RESULT_TYPE {
            ComposableQueryResultKind::Field => {
                return Ok(());
            }
            ComposableQueryResultKind::Selector => fmt.write_str(": ")?,
            ComposableQueryResultKind::FreeObject => fmt.write_str(" := ")?,
        };

        Self::format_selector(fmt)
    }
}

impl<T: EdgedbComposableSelector> EdgedbComposableSelector for Vec<T> {
    const RESULT_TYPE: ComposableQueryResultKind = T::RESULT_TYPE;

    fn format_selector(fmt: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        T::format_selector(fmt)
    }
}

impl EdgedbArgs for () {
    const EDGEDB_TYPE_NAME: Option<&'static str> = None;

    const IS_OPTIONAL: bool = false;

    fn as_query_arg(&self) -> Value {
        todo!()
    }

    fn from_query_result(t: Value) -> Self {
        todo!()
    }
}

impl<T1: EdgedbArgs> EdgedbArgs for (T1,) {
    const EDGEDB_TYPE_NAME: Option<&'static str> = None; // todo

    const IS_OPTIONAL: bool = false;

    fn as_query_arg(&self) -> Value {
        // Value::Tuple(vec![self.0.as_query_arg()])
        self.0.as_query_arg()
    }

    fn from_query_result(t: Value) -> Self {
        todo!()
    }
}

impl<T1: EdgedbArgs, T2: EdgedbArgs> EdgedbArgs for (T1, T2) {
    const EDGEDB_TYPE_NAME: Option<&'static str> = None; // todo

    const IS_OPTIONAL: bool = false;

    fn as_query_arg(&self) -> Value {
        todo!()
    }

    fn from_query_result(t: Value) -> Self {
        todo!()
    }
}

pub async fn query<T: ComposableQuery>(
    client: edgedb_tokio::Client,
    args: T::ArgTypes,
) -> Result<T::ReturnType, edgedb_tokio::Error> {
    let v = client
        .query::<Value, (Value,)>(&T::query(), &(args.as_query_arg(),))
        .await?;

    Ok(T::ReturnType::from_query_result(Value::Set(v)))
}
