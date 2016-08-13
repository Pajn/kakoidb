use std::collections::HashMap;
use std::io::Result;
use datastore::DataStore;

fn new_hash() -> HashMap<String, String> {
    HashMap::new()
}

fn new_list() -> Vec<String> {
    Vec::new()
}

pub struct MemoryDataStore {
    strings: HashMap<String, String>,
    hashes: HashMap<String, HashMap<String, String>>,
    lists: HashMap<String, Vec<String>>,
}

impl MemoryDataStore {
    pub fn new() -> MemoryDataStore {
        MemoryDataStore {strings: HashMap::new(), hashes: HashMap::new(), lists: HashMap::new()}
    }
}

impl DataStore for MemoryDataStore {
    fn get(&self, key: &String) -> Result<Option<&String>> {
        Ok(self.strings.get(key))
    }

    fn set(&mut self, key: &String, value: String) -> Result<()> {
        self.strings.insert(key.clone(), value);
        Ok(())
    }
    fn hget(&self, key: &String, properties: Vec<&String>) -> Result<Option<HashMap<String, Option<String>>>> {
        println!("hget {}, {:?}", key, properties);

        Ok(match self.hashes.get(key) {
            Some(h) => Some(
                properties
                    .into_iter()
                    .map(|property| (property.to_owned(), h.get(property).map(|s| s.clone())))
                    .collect()
            ),
            None => None
        })
    }

    fn hget_all(&self, key: &String) -> Result<Option<HashMap<String, String>>> {
        println!("hget_all {}", key);

        Ok(match self.hashes.get(key) {
            Some(h) => {
                let mut hash = HashMap::new();
                for (property, value) in h.iter() {
                    hash.insert(property.clone(), value.clone());
                }
                Some(hash)
            }
            None => None
        })
    }

    fn hset(&mut self, key: &String, property: &String, value: String) -> Result<()> {
        println!("hset {}, {}, {}", key, property, value);

        let hash = self.hashes.entry(key.clone()).or_insert_with(new_hash);
        hash.insert(property.clone(), value);
        Ok(())
    }

    fn hset_all(&mut self, key: &String, values: HashMap<String, String>) -> Result<()> {
        println!("hset_all {}, {:?}", key, values);

        let hash = self.hashes.entry(key.clone()).or_insert_with(new_hash);
        hash.extend(values);
        Ok(())
    }

    fn lget(&self, key: &String) -> Result<Option<Vec<String>>> {
        println!("lget {}", key);

        Ok(self.lists.get(key).map(Clone::clone))
    }

    fn lpush(&mut self, key: &String, values: Vec<String>) -> Result<()> {
        println!("lpush {}", key);

        let list = self.lists.entry(key.clone()).or_insert_with(new_list);
        list.extend(values);
        Ok(())
    }
}
