//use super::generic_cache::GenericCache;
use memcache;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct MemcachedCache {
    ttl: u16,
    cache_url: String,
}

impl MemcachedCache {
    pub fn new(cache_url: String, ttl: u16) -> Self {
        MemcachedCache { cache_url, ttl }
    }
    fn get_internal(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let mut client = memcache::Client::connect(self.cache_url.clone())?;
        let result = client.get(key)?;
        Ok(result)
    }
    fn set_internal(&mut self, key: &str, value: &str, expire: u16) -> Result<(), Box<dyn Error>> {
        let mut client = memcache::Client::connect(self.cache_url.clone())?;
        client.set(key, value, expire.into())?;
        Ok(())
    }
    pub fn get(&self, key: &str) -> Option<String> {
        trace!("GET {}", key);
        let result = self.get_internal(key);
        match result {
            Ok(result) => result,
            Err(err) => {
                error!("Error on get of memcached value: {}", err);
                None
            }
        }
    }
    pub fn set(&mut self, key: &str, value: &str) {
        trace!("SET {}", key);
        let result = self.set_internal(key, value, self.ttl);
        if let Err(err) = result {
            error!("Error on set of memcached value: {}", err);
        }
    }
}
