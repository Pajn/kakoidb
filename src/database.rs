use std::collections::HashMap;
use std::io;
use datastore::DataStore;
use entities::*;
use node::Node;
use value::{Value, ValueResolver, decode_value, encode_properties, encode_value};

pub struct Database<'a> {
    store: &'a mut DataStore,
}

fn root_key() -> String {
    "root".to_string()
}

fn list_key(id: &str) -> String {
    format!("list_{}", id)
}

fn node_key(id: &str) -> String {
    format!("node_{}", id)
}

fn io_err(error: io::Error) -> Error {
    Error::Io(error)
}

fn node_value(result: KakoiResult<Option<Node>>) -> KakoiResult<Value> {
    result.map(|n| n.map_or(Value::Null, Value::Node))
}

trait HashNode {
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

impl<'a> Database<'a> {
    pub fn new(store: &'a mut DataStore) -> Database<'a> {
        Database {store: store}
    }

    pub fn set(&mut self, mut path: Path, value: Value) -> KakoiResult {
        let mut resolver = ValueResolver::new();
        let value = resolver.resolve(value, &path);
        let value = encode_value(&value);

        for list in resolver.lists {
            let values = list.values.iter().map(encode_value).collect();
            try!(self.store.lpush(&list_key(&list.id), values).map_err(io_err));
        }

        for node in resolver.nodes {
            try!(self.store.hset_all(&node_key(&node.id), encode_properties(node)).map_err(io_err));
        }

        if path.len() > 1 {
            return Err(Error::Unknown)
        }

        match path.pop() {
            Some(part) => self.set_value(&root_key(), part, value),
            None => Err(Error::EmptyPath)
        }
    }

    pub fn select(&self, selector: &Selector) -> KakoiResult<Value> {
        self.run_query(None, selector)
    }

    fn run_query(&self, node_id: Option<&str>, selector: &Selector) -> KakoiResult<Value> {
        match selector {
            &Selector::AllFields => node_value(self.get_full_node(node_id)),
            &Selector::Field(ref field) => node_value(self.get_node(node_id, vec![field])),
            &Selector::Traverse(ref field, selector) => {
                let node = try!(self.get_node(node_id, vec![field]));

                match node {
                    Some(mut node) => {
                        node = try!(self.traverse_field(node, field, selector));
                        Ok(Value::Node(node))
                    },
                    None => Ok(Value::Null),
                }
            },
            &Selector::Multi(ref selectors) => {
                let mut all_fields = false;
                let mut fields = Vec::new();
                let mut traverse = HashMap::new();

                for selector in selectors {
                    match selector {
                        &Selector::AllFields => all_fields = true,
                        &Selector::Field(field) => fields.push(field),
                        &Selector::Traverse(field, selector) => {
                            fields.push(field);
                            traverse.insert(field, selector);
                        }
                        &Selector::Multi(_) => return Err(Error::MultiInMulti),
                    }
                }

                let node = try!(
                    if all_fields { self.get_full_node(node_id) }
                    else          { self.get_node(node_id, fields) }
                );

                match node {
                    Some(mut node) => {
                        for (field, selector) in traverse.iter() {
                            node = try!(self.traverse_field(node, field, selector));
                        }
                        Ok(Value::Node(node))
                    }
                    None => Ok(Value::Null),
                }
            },
        }
    }

    fn traverse_field(&self, mut node: Node, field: &str, selector: &Selector) -> KakoiResult<Node> {
        let value = node.properties[field].clone();
        let sub_query = try!(self.traverse_value(&value, selector));
        node.properties.insert(field.to_owned(), sub_query);
        Ok(node)
    }

    fn traverse_value(&self, value: &Value, selector: &Selector) -> KakoiResult<Value> {
        match value {
            &Value::Link {ref node_id} => self.run_query(Some(node_id), selector),
            &Value::ListLink(ref id) => {
                let list = try!(
                    try!(self.get_list(&id))
                        .iter()
                        .map(|value| self.traverse_value(value, selector))
                        .map(|value| {
                            match try!(value) {
                                Value::Node(node) => Ok(node),
                                _ => Err(Error::Unknown),
                            }
                        })
                        .collect()
                );

                Ok(Value::List(list))
            }
            _ => Err(Error::FieldIsNotTraversable),
        }
    }

    fn get_list(&self, id: &str) -> KakoiResult<Vec<Value>> {
        let list = self.store.lget(&list_key(id));

        list
            .map_err(io_err)
            .and_then(|list| list.unwrap_or_else(Vec::new).iter().map(decode_value).collect())

    }

    fn get_node(&self, id: Option<&str>, fields: Vec<&str>) -> KakoiResult<Option<Node>> {
        let key = match id {
            Some(ref id) => node_key(id),
            None => root_key()
        };

        self.store.hget(&key, fields).into_node(id)
    }

    fn get_full_node(&self, id: Option<&str>) -> KakoiResult<Option<Node>> {
        let key = match id {
            Some(ref id) => node_key(id),
            None => root_key()
        };

        self.store.hget_all(&key).into_node(id)
    }

    fn set_value(&mut self, key: &str, path: PathPart, value: String) -> KakoiResult {
        match path {
            PathPart::Field(ref field) => self.store.hset(key, field, value).map_err(io_err),
        }
    }
}
