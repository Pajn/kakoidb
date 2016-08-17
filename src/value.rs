use std::collections::HashMap;
use std::convert::From;
use uuid::Uuid;
use entities::{Error, Path, PathPart, PrimitiveValue};
use node::Node;

pub type KakoiResult<T = ()> = Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Value {
    I64(i64),
    U64(u64),
    F64(f64),
    Boolean(bool),
    String(String),
    Node(Node),
    Link(String),
    List(Vec<Node>),
    ListLink(String),
    Error(String),
    Null,
}

pub struct List {
    pub id: String,
    pub values: Vec<Value>,
}

pub struct ValueResolver {
    pub lists: Vec<List>,
    pub nodes: Vec<Node>,
}

impl ValueResolver {
    pub fn new() -> ValueResolver {
        ValueResolver {lists: Vec::new(), nodes: Vec::new()}
    }

    fn resolve_node(&mut self, node: &mut Node, path: &Path) -> Value {
        let mut properties = HashMap::new();

        for (property, value) in node.properties.drain() {
            let flattened_value;
            {
                let mut sub_path = path.to_vec();
                sub_path.push(PathPart::Field(&property));
                flattened_value = self.resolve(value, &sub_path);
            }

            properties.insert(property, flattened_value);
        }

        self.nodes.push(Node {id: node.id.clone(), properties: properties});

        Value::Link(node.id.clone())
    }

    pub fn resolve(&mut self, value: Value, path: &Path) -> Value {
        match value {
            Value::Node(mut node) => self.resolve_node(&mut node, path),
            Value::List(mut nodes) => {
                let id = Uuid::new_v4().simple().to_string();

                let list = List {
                    id: id.clone(),
                    values: nodes.iter_mut().map(|mut n| self.resolve_node(&mut n, path)).collect(),
                };

                self.lists.push(list);

                Value::ListLink(id)
            },
            _ => value,
        }
    }
}

impl From<PrimitiveValue> for Value {
    fn from(value: PrimitiveValue) -> Self {
        match value {
            PrimitiveValue::I64(num) => Value::I64(num),
            PrimitiveValue::U64(num) => Value::U64(num),
            PrimitiveValue::F64(num) => Value::F64(num),
            PrimitiveValue::Boolean(boolean) => Value::Boolean(boolean),
            PrimitiveValue::String(string) => string.into(),
            PrimitiveValue::Null => Value::Null,
        }
    }
}

impl<'a> From<&'a str> for Value {
    fn from(string: &str) -> Self {
        match string.chars().next() {
            Some(prefixes::LINK) => Value::Link(string[1..].to_string()),
            Some(prefixes::LIST) => Value::ListLink(string[1..].to_string()),
            Some(prefixes::STRING) => Value::String(string[1..].to_string()),
            Some(c) => Value::Error(format!("Invalid initial character {}", c)),
            None => Value::Null,
        }
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        let s: &str = string.as_ref();
        s.into()
    }
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        match value {
            Value::Link(ref node_id) => format!("{}{}", prefixes::LINK, node_id),
            Value::ListLink(ref id) => format!("{}{}", prefixes::LIST, id),
            Value::String(ref string) => format!("{}{}", prefixes::STRING, string),
            _ => panic!("Can't encode value {:?} as a string", value)
        }
    }
}

impl From<Value> for PrimitiveValue {
    fn from(value: Value) -> Self {
        match value {
            Value::I64(num) => PrimitiveValue::I64(num),
            Value::U64(num) => PrimitiveValue::U64(num),
            Value::F64(num) => PrimitiveValue::F64(num),
            Value::Boolean(boolean) => PrimitiveValue::Boolean(boolean),
            Value::Null => PrimitiveValue::Null,
            Value::Link(_) | Value::ListLink(_) | Value::String(_) =>
                PrimitiveValue::String(value.into()),
            _ => panic!("Value {:?} can't be transformed to a primitive value", value),
        }
    }
}

mod prefixes {
    pub const LINK: char = 'L';
    pub const STRING: char = 'S';
    pub const LIST: char = 'l';
}
