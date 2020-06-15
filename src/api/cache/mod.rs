mod builtin;
mod memcached;
mod redis;

use std::sync::Arc;
use std::sync::Mutex;

pub enum GenericCacheType {
    None,
    BuiltIn,
    Redis,
    Memcached,
}

#[derive(Debug, Clone)]
pub enum GenericCache {
    None,
    BuiltIn(Arc<Mutex<builtin::BuiltinCache>>),
    Redis(redis::RedisCache),
    Memcached(memcached::MemcachedCache),
}

impl GenericCache {
    pub fn new(cache_type: GenericCacheType, cache_url: String, ttl: u16) -> Self {
        match cache_type {
            GenericCacheType::None => GenericCache::None,
            GenericCacheType::BuiltIn => {
                GenericCache::BuiltIn(Arc::new(Mutex::new(builtin::BuiltinCache::new(ttl))))
            }
            GenericCacheType::Redis => GenericCache::Redis(redis::RedisCache::new(cache_url, ttl)),
            GenericCacheType::Memcached => {
                GenericCache::Memcached(memcached::MemcachedCache::new(cache_url, ttl))
            }
        }
    }
    pub fn set(&mut self, key: &str, value: &str) {
        match self {
            GenericCache::None => {}
            GenericCache::BuiltIn(builtin) => {
                let builtin_locked = builtin.lock();
                match builtin_locked {
                    Ok(mut builtin) => {
                        builtin.set(key, value);
                    }
                    Err(err) => {
                        error!("Unable to lock builtin cache: {}", err);
                    }
                }
            }
            GenericCache::Redis(cache) => {
                cache.set(key, value);
            }
            GenericCache::Memcached(cache) => {
                cache.set(key, value);
            }
        };
    }
    pub fn get(&self, key: &str) -> Option<String> {
        match self {
            GenericCache::None => None,
            GenericCache::BuiltIn(builtin) => {
                let builtin_locked = builtin.lock();
                match builtin_locked {
                    Ok(builtin) => builtin.get(key),
                    Err(err) => {
                        error!("Unable to lock builtin cache: {}", err);
                        None
                    }
                }
            }
            GenericCache::Redis(cache) => cache.get(key),
            GenericCache::Memcached(cache) => cache.get(key),
        }
    }
    pub fn cleanup(&mut self) {
        if let GenericCache::BuiltIn(builtin) = self {
            let builtin_locked = builtin.lock();
            match builtin_locked {
                Ok(mut builtin) => builtin.cleanup(),
                Err(err) => {
                    error!("Unable to lock builtin cache: {}", err);
                }
            }
        }
    }
    pub fn needs_cleanup(&self) -> bool {
        if let GenericCache::BuiltIn(_) = self {
            return true;
        }
        return false;
    }
}
