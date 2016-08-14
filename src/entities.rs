use std::cmp::PartialEq;
use std::io;
use value::Value;

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
    String(String),
    Link(String),
    Null,
}

#[derive(Clone, Debug)]
pub enum Predicate<'a> {
    Eq(&'a str, PrimitiveValue),
    Neq(&'a str, PrimitiveValue),
    All(&'a [Predicate<'a>]),
    Any(&'a [Predicate<'a>]),
}

impl PartialEq<PrimitiveValue> for Value {
    fn eq(&self, other: &PrimitiveValue) -> bool {
        match self {
            &Value::String(ref string) => match other {
                &PrimitiveValue::String(ref other) => string == other,
                _ => false,
            },
            &Value::Link(ref id) => match other {
                &PrimitiveValue::Link(ref other) => id == other,
                _ => false,
            },
            &Value::Null => match other {
                &PrimitiveValue::Null => true,
                _ => false,
            },
            _ => false,
        }
    }
}
