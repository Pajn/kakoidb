use std::collections::HashMap;
use datastore::DataStore;
use entities::*;
use keys::*;
use node::Node;
use node::hashnode::{HashNode};
use value::{Value, ValueResolver};

fn match_predicate(predicate: &Predicate, node: &Node) -> bool {
    match predicate {
        &Predicate::All(predicates) => predicates.iter().all(|p| match_predicate(p, node)),
        &Predicate::Any(predicates) => predicates.iter().any(|p| match_predicate(p, node)),
        &Predicate::Eq(field, ref value) => &node.properties[field] == value,
        &Predicate::Neq(field, ref value) => &node.properties[field] != value,
        &Predicate::Lt(field, ref value) => &node.properties[field] < value,
        &Predicate::Lte(field, ref value) => &node.properties[field] <= value,
        &Predicate::Gt(field, ref value) => &node.properties[field] > value,
        &Predicate::Gte(field, ref value) => &node.properties[field] >= value,
    }
}

fn node_value(result: KakoiResult<Option<Node>>) -> KakoiResult<Value> {
    result.map(|n| n.map_or(Value::Null, Value::Node))
}

pub struct Database<'a> {
    store: &'a mut DataStore,
}

impl<'a> Database<'a> {
    pub fn new(store: &'a mut DataStore) -> Database<'a> {
        Database {store: store}
    }

    pub fn set(&mut self, mut path: Path, value: Value) -> KakoiResult {
        let mut resolver = ValueResolver::new();
        let value = resolver.resolve(value, &path).into();

        for list in resolver.lists {
            let values = list.values.into_iter().map(|v| v.into()).collect();
            try!(self.store.lpush(&list_key(&list.id), values).map_err(Error::Io));
        }

        for node in resolver.nodes {
            try!(self.store.hset_all(&node_key(&node.id), node.into()).map_err(Error::Io));
        }

        if path.len() > 1 {
            return Err(Error::Unknown)
        }

        match path.pop() {
            Some(part) => self.set_value(&root_key(), part, value),
            None => Err(Error::EmptyPath)
        }
    }

    pub fn select(&self, selector: &Selector) -> KakoiResult<HashMap<String, Value>> {
        let root_node = try!(self.run_query(None, selector));

        match root_node {
            Value::Node(node) => Ok(node.properties),
            _ => panic!("No root node returned"),
        }
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

    fn traverse_field(&self, mut node: Node, field: &str, selector: &FilteredSelector) -> KakoiResult<Node> {
        let value = node.properties[field].clone();
        let sub_query = try!(self.traverse_value(&value, selector));
        node.properties.insert(field.to_owned(), sub_query);
        Ok(node)
    }

    fn traverse_value<'b>(&self, value: &Value, selector: &'b FilteredSelector) -> KakoiResult<Value> {
        match value {
            &Value::Link(ref node_id) => self.run_query(Some(node_id), &selector.selector),
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
                        .filter(|result: &KakoiResult<Node>| {
                            let node = match result {
                                &Ok(ref node) => node,
                                &Err(_) => return true,
                            };

                            let filter = &selector.filter.to_owned();
                            filter.as_ref().map_or(true, |p| match_predicate(p, node))
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
            .map_err(Error::Io)
            .map(|list| list
                .unwrap_or_else(Vec::new)
                .into_iter()
                .map(From::from)
                .collect())

    }

    fn get_node(&self, id: Option<&str>, fields: Vec<&str>) -> KakoiResult<Option<Node>> {
        let key = match id {
            Some(ref id) => node_key(id),
            None => root_key()
        };

        self.store.hget(&key, fields)
            .map_err(Error::Io)
            .into_node(id)
    }

    fn get_full_node(&self, id: Option<&str>) -> KakoiResult<Option<Node>> {
        let key = match id {
            Some(ref id) => node_key(id),
            None => root_key()
        };

        self.store.hget_all(&key)
            .map_err(Error::Io)
            .into_node(id)
    }

    fn set_value(&mut self, key: &str, path: PathPart, value: PrimitiveValue) -> KakoiResult {
        match path {
            PathPart::Field(ref field) => self.store.hset(key, field, value).map_err(Error::Io),
        }
    }
}
