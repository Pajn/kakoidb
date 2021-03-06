pub mod hashnode;

use std::collections::HashMap;
use entities::KakoiResult;
use value::Value;

pub type NodeProperties = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: String,
    pub properties: NodeProperties,
}

impl Node {
    pub fn new<T>(id: String, properties: HashMap<String, T>) -> KakoiResult<Node> where
        T: Into<Value> {

        let properties = properties
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect();

        Ok(Node {id: id, properties: properties})
    }
}

impl<T> Into<HashMap<String, T>> for Node where T: From<Value> {
    fn into(self) -> HashMap<String, T> {
        self.properties
            .into_iter()
            .map(|(field, value)| (field, value.into()))
            .collect()
    }
}
