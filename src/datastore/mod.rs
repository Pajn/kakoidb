pub mod memory;

use std::collections::HashMap;
use std::io::Result;
use entities::PrimitiveValue;

pub trait DataStore {
    fn get(&self, key: &str) -> Result<&PrimitiveValue>;
    fn set(&mut self, key: &str, value: PrimitiveValue) -> Result<()>;

    fn hget(&self, key: &str, properties: Vec<&str>) -> Result<Option<HashMap<String, PrimitiveValue>>>;
    fn hget_all(&self, key: &str) -> Result<Option<HashMap<String, PrimitiveValue>>>;
    fn hset(&mut self, key: &str, property: &str, value: PrimitiveValue) -> Result<()>;
    fn hset_all(&mut self, key: &str, values: HashMap<String, PrimitiveValue>) -> Result<()>;

    fn lget(&self, key: &str) -> Result<Option<Vec<PrimitiveValue>>>;
    fn lpush(&mut self, key: &str, values: Vec<PrimitiveValue>) -> Result<()>;
}

//abstract class DataStore {
//Future get(String key);
//Future set(String key, value);
//Future del(String key);
//
//Future hget(String key, String property);
//Future<Map<String, dynamic>> hgetAll(String key, [Iterable<String> properties]);
//Future hset(String key, String property, value);
//Future hsetAll(String key, Map<String, dynamic> values);
//
////  Future lpush(String key, Iterable<String> values);
//
//Future sadd(String key, String member);
//Future saddAll(String key, Iterable<String> members);
//Future shas(String key, String member);
//Future srem(String key, String member);
//Future sremAll(String key, Iterable<String> members);
//
//Future zadd(String key, String member, num score);
//Future zaddAll(String key, Map<String, num> members);
//Future zrem(String key, String member);
//Future zremAll(String key, Iterable<String> members);
//Future zrange(String key, {
//num min: double.NEGATIVE_INFINITY,
//num max: double.INFINITY,
//Order order: Order.asc,
//num offset, num count});
//}
