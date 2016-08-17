use std::collections::HashMap;
use entities::KakoiResult;
use node::Node;
use value::Value;

fn unwrap_id(id: Option<&str>) -> String {
    id.map(|id| id.to_owned()).unwrap_or("".to_string())
}

pub trait HashNode {
    fn into_node(self, id: Option<&str>) -> KakoiResult<Option<Node>>;
}

impl<T> HashNode for KakoiResult<Option<HashMap<String, T>>> where T: Into<Value> {
    fn into_node(self, id: Option<&str>) -> KakoiResult<Option<Node>> {
        self
            .and_then(|hash| {
                match hash {
                    Some(props) => Node::new(unwrap_id(id), props).map(Some),
                    None => Ok(None),
                }
            })
    }
}
