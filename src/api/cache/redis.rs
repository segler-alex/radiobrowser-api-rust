//use super::generic_cache::GenericCache;
use redis;
use redis::Commands;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct RedisCache {
    ttl: u16,
    cache_url: String,
}

impl RedisCache {
    pub fn new(cache_url: String, ttl: u16) -> Self {
        RedisCache { cache_url, ttl }
    }
    fn get_internal(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let client = redis::Client::open(self.cache_url.clone())?;
        let mut con = client.get_connection()?;
        let result = con.get(key);
        match result {
            Ok(result) => Ok(Some(result)),
            Err(_) => Ok(None),
        }
    }
    fn set_internal(&mut self, key: &str, value: &str, expire: u16) -> Result<(), Box<dyn Error>> {
        let client = redis::Client::open(self.cache_url.clone())?;
        let mut con = client.get_connection()?;
        let expire: usize = expire.into();
        con.set_ex(key, value, expire)?;
        Ok(())
    }
    pub fn get(&self, key: &str) -> Option<String> {
        trace!("GET {}", key);
        let result = self.get_internal(key);
        match result {
            Ok(result) => result,
            Err(err) => {
                error!("Error on get of redis value: {}", err);
                None
            }
        }
    }
    pub fn set(&mut self, key: &str, value: &str) {
        trace!("SET {}", key);
        let result = self.set_internal(key, value, self.ttl);
        if let Err(err) = result {
            error!("Error on set of redis value: {}", err);
        }
    }
}
