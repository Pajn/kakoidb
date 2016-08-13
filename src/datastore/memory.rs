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
    fn get(&self, key: &str) -> Result<Option<&String>> {
        Ok(self.strings.get(key))
    }

    fn set(&mut self, key: &str, value: String) -> Result<()> {
        self.strings.insert(key.to_string(), value);
        Ok(())
    }
    fn hget(&self, key: &str, properties: Vec<&str>) -> Result<Option<HashMap<String, Option<String>>>> {
        debug!("hget {}, {:?}", key, properties);

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

    fn hget_all(&self, key: &str) -> Result<Option<HashMap<String, String>>> {
        debug!("hget_all {}", key);

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

    fn hset(&mut self, key: &str, property: &str, value: String) -> Result<()> {
        debug!("hset {}, {}, {}", key, property, value);

        let hash = self.hashes.entry(key.to_string()).or_insert_with(new_hash);
        hash.insert(property.to_string(), value);
        Ok(())
    }

    fn hset_all(&mut self, key: &str, values: HashMap<String, String>) -> Result<()> {
        debug!("hset_all {}, {:?}", key, values);

        let hash = self.hashes.entry(key.to_string()).or_insert_with(new_hash);
        hash.extend(values);
        Ok(())
    }

    fn lget(&self, key: &str) -> Result<Option<Vec<String>>> {
        debug!("lget {}", key);

        Ok(self.lists.get(key).map(Clone::clone))
    }

    fn lpush(&mut self, key: &str, values: Vec<String>) -> Result<()> {
        debug!("lpush {}", key);

        let list = self.lists.entry(key.to_string()).or_insert_with(new_list);
        list.extend(values);
        Ok(())
    }
}
