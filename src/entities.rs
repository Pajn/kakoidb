use std::io;
use predicate::Predicate;

#[derive(Clone)]
pub enum PathPart<'a> {
    Field(&'a str),
}

pub type Path<'a> = Vec<PathPart<'a>>;

#[derive(Debug)]
pub enum Error {
    EmptyPath,
    FieldIsNotTraversable,
    InvalidValue,
    Io(io::Error),
    MultiInMulti,
    Unknown,
}

pub type KakoiResult<T = ()> = Result<T, Error>;

#[derive(Debug)]
pub enum Selector<'a> {
    AllFields,
    Field(&'a str),
    Multi(Vec<Selector<'a>>),
    Traverse(&'a str, &'a FilteredSelector<'a>),
}

#[derive(Debug)]
pub struct FilteredSelector<'a> {
    pub selector: Selector<'a>,
    pub filter: Option<Predicate<'a>>,
}

#[derive(Clone, Debug)]
pub enum PrimitiveValue {
    I64(i64),
    U64(u64),
    F64(f64),
    Boolean(bool),
    String(String),
    Null,
}

impl<'a> From<&'a str> for PrimitiveValue {
    fn from(value: &str) -> Self {
        PrimitiveValue::String(value.to_string())
    }
}

impl<'a> From<i64> for PrimitiveValue {
    fn from(value: i64) -> Self {
        PrimitiveValue::I64(value)
    }
}
