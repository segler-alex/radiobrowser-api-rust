use std::sync::Mutex;
use std::sync::Arc;
use super::generic_cache::GenericCache;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Item {
    value: String,
    expire: SystemTime,
}

impl Item {
    pub fn new(value: String, expire: u16) -> Self {
        Item {
            value,
            expire: SystemTime::now() + Duration::new(expire.into(), 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinCache {
    ttl: u16,
    cache: Arc<Mutex<HashMap<String, Item>>>,
}

impl BuiltinCache {
    pub fn new(ttl: u16) -> Self {
        BuiltinCache {
            ttl,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl GenericCache for BuiltinCache {
    fn get(&self, key: &str) -> Option<String> {
        trace!("GET {}", key);
        let locked = self.cache.lock();
        match locked {
            Ok(locked) => {
                match locked.get(key.into()) {
                    Some(item) => {
                        let now = SystemTime::now();
                        if item.expire > now {
                            Some(item.value.clone())
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            },
            Err(err) => {
                error!("Unable to lock counter for get: {}", err);
                None
            }
        }
    }
    fn set(&mut self, key: &str, value: &str) {
        trace!("SET {}={}", key, value);
        let locked = self.cache.lock();
        match locked {
            Ok(mut locked) => {
                locked
                    .entry(key.to_string())
                    .or_insert(Item::new(value.into(), self.ttl));
            }
            Err(err) => {
                error!("Unable to lock counter for set: {}", err);
            }
        }
    }
    fn cleanup(&mut self){
        let locked = self.cache.lock();
        match locked {
            Ok(mut locked) => {
                let now = SystemTime::now();
                let mut to_delete: Vec<String> = vec![];
                        
                for (key, value) in locked.iter() {
                    if value.expire <= now {
                        to_delete.push(key.clone());
                    }
                }

                for key in to_delete {
                    locked.remove(&key);
                }
            }
            Err(err) => {
                error!("Unable to lock counter for set: {}", err);
            }
        }
    }
}
