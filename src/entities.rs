use std::io;
use node::NodeProperties;
use predicate::Predicate;
use node::Node;
use value::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum PathPart<'a> {
    Field(&'a str),
    FieldFilter(&'a str, Predicate<'a>),
}

pub type Path<'a> = &'a [PathPart<'a>];

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

#[derive(Clone, Debug, PartialEq)]
pub enum Selector<'a> {
    AllFields,
    Field(&'a str),
    Multi(Vec<Selector<'a>>),
    Traverse(&'a str, &'a FilteredSelector<'a>),
}

impl<'a> Selector<'a> {
    pub fn get_fields(&self) -> Option<Vec<&str>> {
        match self {
            &Selector::AllFields => None,
            &Selector::Field(field) => Some(vec![field]),
            &Selector::Multi(ref selectors) => {
                let mut fields = Vec::new();
                for selector in selectors {
                    match selector.get_fields() {
                        Some(f) => fields.extend(f),
                        None => {return None}
                    }
                }
                Some(fields)
            },
            &Selector::Traverse(field, filter) => {
                filter.get_fields().map(|mut fields| {
                    fields.push(field);
                    fields
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilteredSelector<'a> {
    pub selector: Selector<'a>,
    pub filter: Option<Predicate<'a>>,
}

impl<'a> FilteredSelector<'a> {
    pub fn get_fields(&self) -> Option<Vec<&str>> {
        let mut fields = self.selector.get_fields();

        if let Some(ref filter) = self.filter {
            fields = fields.map(|mut f| {
                f.extend(filter.get_fields());
                f
            });
        }

        fields
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct Mutation<'a> {
    pub path: Path<'a>,
    pub opertaion: MutationOperation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeType {
    Node(Node),
    Nodes(Vec<Node>),
    Link(String),
    Links(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MutationOperation {
    Append(NodeType),
    Set(Value),
    Merge(NodeProperties),
}
