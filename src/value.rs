use std::collections::HashMap;
use uuid::Uuid;
use entities::{Error, KakoiResult, Path, PathPart};
use node::Node;

#[derive(Clone, Debug)]
pub enum Value {
    //    I64(i64),
    //    U64(u64),
    //    F64(f64),
    //    Boolean(bool),
    //    Array(Array),
    String(String),
    Node(Node),
    Link(String),
    List(Vec<Node>),
    ListLink(String),
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

pub fn encode_properties(node: Node) -> HashMap<String, String> {
    let mut properties = HashMap::new();

    for (field, value) in node.properties {
        properties.insert(field, encode_value(&value));
    }

    properties
}

mod prefixes {
    pub const LINK: char = 'L';
    pub const STRING: char = 'S';
    pub const LIST: char = 'l';
}

pub fn encode_value(value: &Value) -> String {
    match value {
        &Value::Link(ref node_id) => format!("{}{}", prefixes::LINK, node_id),
        &Value::ListLink(ref id) => format!("{}{}", prefixes::LIST, id),
        &Value::String(ref string) => format!("{}{}", prefixes::STRING, string),
        _ => panic!("Got value: {:?}", value)
    }
}

pub fn decode_optional_value(string: &Option<String>) -> KakoiResult<Value> {
    match string {
        &Some(ref s) => decode_value(s),
        &None => Ok(Value::Null),
    }
}

pub fn decode_value(string: &String) -> KakoiResult<Value> {
    match string.chars().next() {
        Some(prefixes::LINK) => Ok(Value::Link(string[1..].to_string())),
        Some(prefixes::LIST) => Ok(Value::ListLink(string[1..].to_string())),
        Some(prefixes::STRING) => Ok(Value::String(string[1..].to_string())),
        Some(_) => Err(Error::InvalidValue),
        None => Ok(Value::Null),
    }
}
