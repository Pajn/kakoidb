//#![feature(try_from)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate uuid;

mod datastore;
mod database;
mod entities;
mod keys;
mod node;
mod predicate;
mod value;

use std::collections::HashMap;
use datastore::memory::MemoryDataStore;
use database::Database;
use entities::{FilteredSelector, PathPart, Selector, PrimitiveValue};
use node::Node;
use predicate::Predicate;
use value::Value;


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

fn main() {
    env_logger::init().unwrap();

    let mut store = MemoryDataStore::new();
    let mut db = Database::new(&mut store);

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

    let value = db.select(
        &Selector::Traverse(
            &"series",
            &FilteredSelector {selector: Selector::Multi(vec![
                Selector::Field("name"),
                Selector::Traverse(
                    "episodes",
                    &FilteredSelector {
                        selector: Selector::Field("name"),
                        filter: Some(Predicate::Any(&[
                            Predicate::Eq("name", PrimitiveValue::String("A Study in Pink".to_string())),
                            Predicate::Eq("name", PrimitiveValue::String("Pilot".to_string())),
                        ])),
                    },
                ),
            ]), filter: None},
        )
    ).unwrap();

    println!("value => {:#?}", value);

    let value = db.select(
        &Selector::Traverse(
            "series",
            &FilteredSelector {selector: Selector::AllFields, filter: None},
        )
    ).unwrap();

    println!("value => {:#?}", value);

    let value = db.select(
        &Selector::Traverse(
            "series",
            &FilteredSelector {selector: Selector::Multi(vec![
                Selector::AllFields,
                Selector::Traverse(
                    "episodes",
                    &FilteredSelector {selector: Selector::Field("name"), filter: None},
                ),
            ]), filter: Some(Predicate::All(&[
                Predicate::Gte("year", PrimitiveValue::I64(2012)),
                Predicate::Lt("year", PrimitiveValue::I64(2013)),
            ]))},
        )
    ).unwrap();

    println!("value => {:#?}", value);
}
