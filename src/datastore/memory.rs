use std::collections::HashMap;
use std::io::Result;
use datastore::DataStore;
use entities::PrimitiveValue;

fn new_hash() -> HashMap<String, PrimitiveValue> {
    HashMap::new()
}

fn new_list() -> Vec<PrimitiveValue> {
    Vec::new()
}

pub struct MemoryDataStore {
    values: HashMap<String, PrimitiveValue>,
    hashes: HashMap<String, HashMap<String, PrimitiveValue>>,
    lists: HashMap<String, Vec<PrimitiveValue>>,
    null: PrimitiveValue,
}

impl MemoryDataStore {
    pub fn new() -> MemoryDataStore {
        MemoryDataStore {
            values: HashMap::new(),
            hashes: HashMap::new(),
            lists: HashMap::new(),
            null: PrimitiveValue::Null,
        }
    }
}

impl DataStore for MemoryDataStore {
    fn get(&self, key: &str) -> Result<&PrimitiveValue> {
        Ok(self.values.get(key).unwrap_or(&self.null))
    }

    fn set(&mut self, key: &str, value: PrimitiveValue) -> Result<()> {
        self.values.insert(key.to_string(), value);
        Ok(())
    }
    fn hget(&self, key: &str, properties: Vec<&str>) -> Result<Option<HashMap<String, PrimitiveValue>>> {
        debug!("hget {}, {:?}", key, properties);

        Ok(self.hashes.get(key).map(|h| {
            properties
                .into_iter()
                .map(|property| (property.to_owned(), h.get(property).map_or(PrimitiveValue::Null, |s| s.clone())))
                .collect()
        }))
    }

    fn hget_all(&self, key: &str) -> Result<Option<HashMap<String, PrimitiveValue>>> {
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

    fn hset(&mut self, key: &str, property: &str, value: &PrimitiveValue) -> Result<()> {
        debug!("hset {}, {}, {:?}", key, property, value);

        let hash = self.hashes.entry(key.to_string()).or_insert_with(new_hash);
        hash.insert(property.to_string(), value.to_owned());
        Ok(())
    }

    fn hset_all(&mut self, key: &str, values: &HashMap<String, PrimitiveValue>) -> Result<()> {
        debug!("hset_all {}, {:?}", key, values);

        let hash = self.hashes.entry(key.to_string()).or_insert_with(new_hash);
        hash.extend(values.to_owned());
        Ok(())
    }

    fn lget(&self, key: &str) -> Result<Option<Vec<PrimitiveValue>>> {
        debug!("lget {}", key);

        Ok(self.lists.get(key).map(Clone::clone))
    }

    fn lpush(&mut self, key: &str, values: &Vec<PrimitiveValue>) -> Result<()> {
        debug!("lpush {}", key);

        let list = self.lists.entry(key.to_string()).or_insert_with(new_list);
        list.extend(values.to_owned());
        Ok(())
    }
}
