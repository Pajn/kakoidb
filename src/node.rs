use std::collections::HashMap;
use entities::KakoiResult;
use value::{Value, decode_optional_value};

#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
    pub properties: HashMap<String, Value>,
}

impl Node {
    pub fn new(id: String, properties: HashMap<String, Option<String>>) -> KakoiResult<Node> {
        let decoded_properties = try!(properties
            .into_iter()
            .map(|(key, value)| {
                let decoded_value = try!(decode_optional_value(&value));
                Ok((key, decoded_value))
            })
            .collect()
        );

        Ok(Node {id: id, properties: decoded_properties})
    }
}
