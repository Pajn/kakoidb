#[macro_use]
extern crate log;
extern crate env_logger;
extern crate uuid;

mod datastore;
mod database;
mod entities;
mod node;
mod value;

use std::collections::HashMap;
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
    env_logger::init().unwrap();

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
        vec![PathPart::Field(&"series")],
        Value::List(vec![elementary, sherlock]),
    ).unwrap();

    let value = db.select(
        &Selector::Traverse(
            &"series",
            &Selector::Multi(vec![
                Selector::Field("name"),
                Selector::Traverse(
                    "episodes",
                    &Selector::Field("name"),
                ),
            ]),
        )
    ).unwrap();

    println!("value => {:#?}", value);

    let value = db.select(
        &Selector::Traverse(
            "series",
            &Selector::AllFields,
        )
    ).unwrap();

    println!("value => {:#?}", value);

    let value = db.select(
        &Selector::Traverse(
            "series",
            &Selector::Multi(vec![
                Selector::AllFields,
                Selector::Traverse(
                    "episodes",
                    &Selector::Field("name"),
                ),
            ]),
        )
    ).unwrap();

    println!("value => {:#?}", value);
}
