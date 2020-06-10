mod builtin;
mod generic_cache;
mod memcached;
mod none;
mod redis;

use generic_cache::GenericCache;

#[derive(Debug, Clone)]
pub enum CacheType {
    None,
    BuiltIn,
    Redis,
    Memcached,
}

pub struct CacheConnection {
    pub cache: Box<dyn GenericCache>,
}

impl CacheConnection {
    pub fn new(cache_type: CacheType, cache_url: String, ttl: u16) -> Self {
        let cache: Box<dyn GenericCache> = match cache_type {
            CacheType::None => Box::new(none::NoneCache::new()),
            CacheType::BuiltIn => Box::new(builtin::BuiltinCache::new(ttl)),
            CacheType::Redis => Box::new(redis::RedisCache::new(cache_url, ttl)),
            CacheType::Memcached => Box::new(memcached::MemcachedCache::new(cache_url, ttl)),
        };
        CacheConnection { cache }
    }
}
