pub mod hashnode;

use std::collections::HashMap;
use entities::KakoiResult;
use value::Value;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
    pub properties: HashMap<String, Value>,
}

impl Node {
    pub fn new<T>(id: String, properties: HashMap<String, T>) -> KakoiResult<Node> where
        T: Into<KakoiResult<Value>> {

        let decoded_properties = try!(properties
            .into_iter()
            .map(|(key, value)| {
                let value = try!(value.into());
                Ok((key, value))
            })
            .collect()
        );

        Ok(Node {id: id, properties: decoded_properties})
    }
}
//where entities::Error: std::convert::From<<T as std::convert::TryInto<value::Value>>::Err>
