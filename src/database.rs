use std::collections::HashMap;
use datastore::DataStore;
use entities::*;
use keys::*;
use node::{Node, NodeProperties};
use node::hashnode::{HashNode};
use predicate::{MatchesPredicate, Predicate};
use value::{Value, ValueResolver};

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

    pub fn mutate(&mut self, mutation: Mutation) -> KakoiResult {
        match mutation.opertaion {
            MutationOperation::Append(node) => self.append(mutation.path, node),
            MutationOperation::Merge(properties) => self.merge(mutation.path, properties),
            MutationOperation::Set(value) => self.set(mutation.path, value),
        }
    }

    pub fn append(&mut self, path: Path, node: NodeType) -> KakoiResult {
        let keys = try!(self.resolve_path(path, true));
        println!("keys {:?}", keys);

        let node_values = match node {
            NodeType::Node(node) => vec![Value::Node(node)],
            NodeType::Nodes(nodes) => nodes.into_iter().map(Value::Node).collect(),
            NodeType::Link(id) => vec![Value::Link(id)],
            NodeType::Links(ids) => ids.into_iter().map(Value::Link).collect(),
        };
        println!("node_values {:?}", node_values);

        let values = try!(node_values
            .into_iter()
            .map(|node| self.resolve_value(&path, node))
            .collect()
        );
        println!("values {:?}", values);

        for key in keys {
            try!(self.store.lpush(&key, &values)
                .map_err(Error::Io));
        }

        Ok(())
    }

    pub fn merge(&mut self, path: Path, properties: NodeProperties) -> KakoiResult {
        let keys = try!(self.resolve_path(path, false));

        for key in keys {
            try!(self.store.hset_all(&key, &properties.clone().into_iter().map(|(k, v)| (k, v.into())).collect())
                .map_err(Error::Io));
        }

        Ok(())
    }

    pub fn set(&mut self, path: Path, value: Value) -> KakoiResult {
        let value = try!(self.resolve_value(&path, value));

        let final_part = path.last();
        let keys = try!(self.resolve_path(path, false));

        for key in keys {
            try!(self.set_value(&key, &final_part.unwrap(), &value));
        }

        Ok(())
    }

    fn resolve_value(&mut self, path: &Path, value: Value) -> KakoiResult<PrimitiveValue> {
        let mut resolver = ValueResolver::new();
        let value: PrimitiveValue = resolver.resolve(value, path).into();

        for list in resolver.lists {
            let values = list.values.into_iter().map(|v| v.into()).collect();
            try!(self.store.lpush(&list_key(&list.id), &values).map_err(Error::Io));
        }

        for node in resolver.nodes {
            try!(self.store.hset_all(&node_key(&node.id), &node.into()).map_err(Error::Io));
        }

        Ok(value)
    }

    fn resolve_path(&self, path: Path, return_list: bool) -> KakoiResult<Vec<String>> {
        if path.is_empty() { return Err(Error::EmptyPath) }

        let mut keys = vec![root_key()];
        let mut index = 0;

        fn get_field<'b>(part: &'b PathPart) -> (&'b str, Option<&'b Predicate<'b>>) {
            match part {
                &PathPart::Field(field) => (field, None),
                &PathPart::FieldFilter(field, ref filter) => (field, Some(filter)),
            }
        }

        while index < path.len() {
            let ref part = path[index];
            let (field, filter) = get_field(part);

            let mut next_keys = Vec::new();

            for key in &keys {
                let value: Value = try!(self.get_field(&key, field)).into();

                match value {
                    Value::Link(id) => next_keys.push(node_key(&id)),
                    Value::ListLink(id) => {
                        if return_list && index == path.len() - 1 {
                            next_keys.push(list_key(&id));
                            continue;
                        }
                        match filter {
                            Some(filter) => {
                                let fields = filter.get_fields();
                                let nodes: Vec<String> =
                                    try!(self.get_filtered_list(&id, Some(&fields), &Some(filter)))
                                    .iter()
                                    .map(|n| node_key(&n.id))
                                    .collect();

                                next_keys.extend(nodes);

                            }
                            None => {
                                let nodes: Vec<String> = try!(self.get_list(&id))
                                    .iter()
                                    .filter_map(|v| {
                                        if let &Value::Link(ref id) = v {
                                            Some(node_key(id))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                next_keys.extend(nodes);
                            }
                        }
                    },
                    _ => {
                        if index < path.len() - 1 {
                            return Err(Error::Unknown)
                        } else {
                            return Ok(keys.clone())
                        }
                    },
                };
            }

            keys = next_keys;
            index += 1;
        }


        Ok(keys)
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
                let fields = selector.get_fields();
                let fields: Option<&[&str]> = match fields {
                    Some(ref fields) => Some(fields),
                    None => None,
                };
                let list = try!(self.get_filtered_list(id, fields, &selector.filter.as_ref()));

                if let Selector::Traverse(field, selector) = selector.selector {
                    let list: KakoiResult<Vec<Node>> = list
                        .into_iter()
                        .map(|mut node| {
                            let traversed_value = try!(self.traverse_value(&node.properties[field], selector));
                            node.properties.insert(field.to_string(), traversed_value);
                            Ok(node)
                        })
                        .collect();

                    return list.map(Value::List);
                }

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

    fn get_filtered_list(&self, id: &str, fields: Option<&[&str]>, filter: &Option<&Predicate>) -> KakoiResult<Vec<Node>> {
        let list = try!(
            try!(self.get_list(id))
                .iter()
                .map(|value| {
                    match value {
                        &Value::Link(ref id) => match fields {
                            Some(fields) => self.get_node(Some(id), fields.to_owned()),
                            None => self.get_full_node(Some(id)),
                        },
                        _ => Ok(None),
                    }
                })
                .filter_map(|result| {
                    let node = match result {
                        Ok(Some(node)) => node,
                        Ok(None) => return None,
                        Err(err) => return Some(Err(err)),
                    };

                    if filter.as_ref().map_or(true, |p| node.matches(p)) {
                        Some(Ok(node))
                    } else {
                        None
                    }
                })
                .collect()
        );

        Ok(list)
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

    fn get_field(&self, key: &str, field: &str) -> KakoiResult<PrimitiveValue> {
        self.store.hget(key, vec![field])
            .map_err(Error::Io)
            .map(|props| props.and_then(|mut p| p.remove(field)).unwrap_or(PrimitiveValue::Null))
    }

    fn set_value(&mut self, key: &str, path: &PathPart, value: &PrimitiveValue) -> KakoiResult {
        match path {
            &PathPart::Field(ref field) | &PathPart::FieldFilter(ref field, _) =>
                self.store.hset(key, field, value).map_err(Error::Io),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use datastore::memory::MemoryDataStore;
    use entities::{FilteredSelector, KakoiResult, Mutation, MutationOperation, NodeType, PathPart, Selector};
    use node::Node;
    use predicate::Predicate;
    use value::Value;

    fn names(series: Vec<Node>) -> Vec<String> {
        series
            .iter()
            .map(|n| n.properties.get("name").unwrap())
            .map(|value| match value {
                &Value::String(ref name) => name.to_owned(),
                _ => panic!("Name not a String, got {:?}", value),
            })
            .collect()
    }

    fn serie(name: &str, year: i64, episodes: Vec<Node>) -> Node {
        let mut node = Node {id: name.to_string(), properties: HashMap::new()};
        node.properties.insert("name".to_string(), Value::String(name.to_string()));
        node.properties.insert("year".to_string(), Value::I64(year));
        node.properties.insert("episodes".to_string(), Value::List(episodes));
        node
    }

    fn episode(name: &str) -> Node {
        let mut node = Node {id: name.to_string(), properties: HashMap::new()};
        node.properties.insert("name".to_string(), Value::String(name.to_string()));
        node
    }

    fn create_db<'a>(store: &'a mut MemoryDataStore) -> Database<'a> {
        let mut db = Database::new(store);

        let elementary = serie(
            "Elementary",
            2012,
            vec![
                episode("Pilot"),
                episode("While You Were Sleeping"),
            ],
        );

        let sherlock = serie(
            "Sherlock",
            2010,
            vec![
                episode("A Study in Pink"),
                episode("The Blind Banker"),
            ],
        );

        db.set(
            &[PathPart::Field(&"series")],
            Value::List(vec![elementary, sherlock]),
        ).unwrap();

        db
    }

    fn get_series(result: KakoiResult<HashMap<String, Value>>) -> Vec<Node> {
        let list = result.unwrap().remove("series").unwrap();
        if let Value::List(list) = list {
            list
        } else {
            panic!("{:?} where returned for series, expected a List", list)
        }
    }

    fn get_episodes(node: &mut Node) -> Vec<Node> {
        let list = node.properties.remove("episodes").unwrap();
        if let Value::List(list) = list {
            list
        } else {
            panic!("{:?} where returned for episodes, expected a List", list)
        }
    }

    #[test]
    fn basic_operation() {
        let mut store = MemoryDataStore::new();
        create_db(&mut store);
    }

    #[test]
    fn select() {
        let mut store = MemoryDataStore::new();
        let db = create_db(&mut store);

        let series = get_series(db.select(&Selector::Traverse("series", &FilteredSelector {
            selector: Selector::Field("name"),
            filter: None,
        })));

        assert_eq!(series.len(), 2);
        assert_eq!(names(series), ["Elementary", "Sherlock"]);
    }

    #[test]
    fn select_traverse() {
        let mut store = MemoryDataStore::new();
        let db = create_db(&mut store);

        let mut series = get_series(db.select(&Selector::Traverse("series", &FilteredSelector {
            selector: Selector::Traverse("episodes", &FilteredSelector {
                selector: Selector::Field("name"),
                filter: None,
            }),
            filter: None,
        })));

        assert_eq!(series.len(), 2);

        let episodes = get_episodes(&mut series[0]);

        assert_eq!(episodes.len(), 2);
        assert_eq!(names(episodes), ["Pilot", "While You Were Sleeping"]);
    }

    #[test]
    fn set() {
        let mut store = MemoryDataStore::new();
        let mut db = create_db(&mut store);

        db.mutate(Mutation {
            path: &[
                PathPart::FieldFilter("series", Predicate::Eq("name", "Sherlock".into())),
                PathPart::Field("episodes"),
                PathPart::Field("name"),
            ],
            opertaion: MutationOperation::Set(Value::String("Name".into())),
        }).unwrap();

        let mut series = get_series(db.select(&Selector::Traverse("series", &FilteredSelector {
            selector: Selector::Traverse("episodes", &FilteredSelector {
                selector: Selector::Field("name"),
                filter: None,
            }),
            filter: None,
        })));

        assert_eq!(series.len(), 2);

        println!("Elementary, Not changed");
        let episodes = get_episodes(&mut series[0]);

        assert_eq!(episodes.len(), 2);
        assert_eq!(names(episodes), ["Pilot", "While You Were Sleeping"]);

        println!("Sherlock, Changed");
        let episodes = get_episodes(&mut series[1]);

        assert_eq!(episodes.len(), 2);
        assert_eq!(names(episodes), ["Name", "Name"]);
    }

    #[test]
    fn merge() {
        let mut store = MemoryDataStore::new();
        let mut db = create_db(&mut store);

        let mut holmes = HashMap::new();
        holmes.insert("name".to_string(), Value::String("Holmes".into()));

        db.mutate(Mutation {
            path: &[PathPart::FieldFilter("series", Predicate::Eq("name", "Sherlock".into()))],
            opertaion: MutationOperation::Merge(holmes),
        }).unwrap();

        let series = get_series(db.select(&Selector::Traverse("series", &FilteredSelector {
            selector: Selector::Field("name"),
            filter: None,
        })));

        assert_eq!(series.len(), 2);
        assert_eq!(names(series), ["Elementary", "Holmes"]);
    }

    #[test]
    fn append() {
        let mut store = MemoryDataStore::new();
        let mut db = create_db(&mut store);

        let sherlock_homes = serie("Sherlock Holmes", 1984, Vec::new());

        db.mutate(Mutation {
            path: &[PathPart::Field("series")],
            opertaion: MutationOperation::Append(NodeType::Node(sherlock_homes)),
        }).unwrap();

        let series = get_series(db.select(&Selector::Traverse("series", &FilteredSelector {
            selector: Selector::Field("name"),
            filter: None,
        })));

        assert_eq!(series.len(), 3);
        assert_eq!(names(series), ["Elementary", "Sherlock", "Sherlock Holmes"]);
    }
}
