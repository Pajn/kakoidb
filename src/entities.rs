use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::io;
use predicate::Predicate;
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
    I64(i64),
    U64(u64),
    F64(f64),
    Boolean(bool),
    String(String),
    Null,
}

impl PartialEq<PrimitiveValue> for Value {
    fn eq(&self, other: &PrimitiveValue) -> bool {
        match self {
            &Value::I64(ref value) => match other {
                &PrimitiveValue::I64(ref other) => value == other,
                &PrimitiveValue::U64(ref other) => *value == *other as i64,
                &PrimitiveValue::F64(ref other) => *value == *other as i64,
                _ => false,
            },
            &Value::U64(ref value) => match other {
                &PrimitiveValue::I64(ref other) => *value == *other as u64,
                &PrimitiveValue::U64(ref other) => value == other,
                &PrimitiveValue::F64(ref other) => *value == *other as u64,
                _ => false,
            },
            &Value::F64(ref value) => match other {
                &PrimitiveValue::I64(ref other) => *value == *other as f64,
                &PrimitiveValue::U64(ref other) => *value == *other as f64,
                &PrimitiveValue::F64(ref other) => value == other,
                _ => false,
            },
            &Value::Boolean(ref value) => match other {
                &PrimitiveValue::Boolean(ref other) => value == other,
                _ => false,
            },
            &Value::String(ref value) => match other {
                &PrimitiveValue::String(ref other) => value == other,
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

impl PartialOrd<PrimitiveValue> for Value {
    fn partial_cmp(&self, other: &PrimitiveValue) -> Option<Ordering> {
        match self {
            &Value::I64(ref num) => match other {
                &PrimitiveValue::I64(ref other) => num.partial_cmp(other),
                &PrimitiveValue::U64(ref other) => num.partial_cmp(&(*other as i64)),
                &PrimitiveValue::F64(ref other) => num.partial_cmp(&(*other as i64)),
                _ => None,
            },
            &Value::U64(ref num) => match other {
                &PrimitiveValue::I64(ref other) => num.partial_cmp(&(*other as u64)),
                &PrimitiveValue::U64(ref other) => num.partial_cmp(other),
                &PrimitiveValue::F64(ref other) => num.partial_cmp(&(*other as u64)),
                _ => None,
            },
            &Value::F64(ref num) => match other {
                &PrimitiveValue::I64(ref other) => num.partial_cmp(&(*other as f64)),
                &PrimitiveValue::U64(ref other) => num.partial_cmp(&(*other as f64)),
                &PrimitiveValue::F64(ref other) => num.partial_cmp(other),
                _ => None,
            },
            _ => None,
        }
    }
}
