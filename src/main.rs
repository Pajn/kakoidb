extern crate uuid;

mod datastore;
mod database;
mod entities;
mod node;
mod value;

use std::collections::HashMap;
//use datastore::DataStore;
use datastore::memory::MemoryDataStore;
use database::Database;
use entities::PathPart;
use entities::Selector;
use node::Node;
use value::Value;

fn serie(name: &str, episodes: Vec<Node>) -> Node {
    let mut node = Node {id: name.to_string(), properties: HashMap::new()};
    node.properties.insert("name".to_string(), Value::String(name.to_string()));
    node.properties.insert("episodes".to_string(), Value::List(episodes));
    node
}

fn episode(name: &str) -> Node {
    let mut node = Node {id: name.to_string(), properties: HashMap::new()};
    node.properties.insert("name".to_string(), Value::String(name.to_string()));
    node
}

fn main() {
    let mut store = MemoryDataStore::new();
    let mut db = Database::new(&mut store);

    let elementary = serie(
        "Elementary",
        vec![
            episode("Pilot"),
            episode("While You Were Sleeping"),
        ],
    );

    let sherlock = serie(
        "Sherlock",
        vec![
            episode("A Study in Pink"),
            episode("The Blind Banker"),
        ],
    );

    db.set(
        vec![PathPart::Field(&"series".to_string())],
        Value::List(vec![elementary, sherlock]),
    ).unwrap();

    let value = db.select(
        &Selector::Traverse(
            "series".to_string(),
            &Selector::Multi(vec![
                Selector::Field("name".to_string()),
                Selector::Traverse(
                    "episodes".to_string(),
                    &Selector::Field("name".to_string()),
                ),
            ]),
        )
    ).unwrap();

    println!("value => {:#?}", value);

    let value = db.select(
        &Selector::Traverse(
            "series".to_string(),
            &Selector::AllFields,
        )
    ).unwrap();

    println!("value => {:#?}", value);

    let value = db.select(
        &Selector::Traverse(
            "series".to_string(),
            &Selector::Multi(vec![
                Selector::AllFields,
                Selector::Traverse(
                    "episodes".to_string(),
                    &Selector::Field("name".to_string()),
                ),
            ]),
        )
    ).unwrap();

    println!("value => {:#?}", value);

//    store.hset("b".to_string(), "h".to_string(), "1".to_string()).unwrap();
//
//    let mut map = HashMap::new();
//    map.insert("a".to_string(), "1".to_string());
//    map.insert("b".to_string(), "2".to_string());
//    map.insert("c".to_string(), "3".to_string());
//
//    store.hset_all("b".to_string(), map).unwrap();
//
//    let hash = store.hget_all("b".to_string()).unwrap().unwrap();
////    let hash = store.hget("b".to_string(), vec!["a".to_string(), "b".to_string()]).unwrap().unwrap();
//
//    for (key, value) in hash.iter() {
////        println!("{} => {}", key, value.as_ref().unwrap_or(&"__null__".to_string()));
//        println!("{} => {}", key, value);
//    }
}
