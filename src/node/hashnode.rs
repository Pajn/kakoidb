use std::collections::HashMap;
use std::io;
use entities::{Error, KakoiResult};
use node::Node;

pub fn io_err(error: io::Error) -> Error {
    Error::Io(error)
}

pub trait HashNode {
    fn into_node(self, id: Option<&str>) -> KakoiResult<Option<Node>>;
}

impl HashNode for io::Result<Option<HashMap<String, Option<String>>>> {
    fn into_node(self, id: Option<&str>) -> KakoiResult<Option<Node>> {
        self
            .map_err(io_err)
            .and_then(|hash| {
                match hash {
                    Some(props) => {
                        let id = id.map(|id| id.to_owned()).unwrap_or("root".to_string());
                        Node::new(id, props).map(Some)
                    },
                    None => Ok(None),
                }
            })
    }
}

impl HashNode for io::Result<Option<HashMap<String, String>>> {
    fn into_node(self, id: Option<&str>) -> KakoiResult<Option<Node>> {
        self
            .map_err(io_err)
            .and_then(|hash| {
                match hash {
                    Some(props) => {
                        let id = id.map(|id| id.to_owned()).unwrap_or("root".to_string());
                        let option_props = props
                            .into_iter()
                            .map(|(key, value)| (key, Some(value)))
                            .collect();
                        Node::new(id, option_props).map(Some)
                    },
                    None => Ok(None),
                }
            })
    }
}
