use super::generic_cache::GenericCache;

#[derive(Debug, Clone)]
pub struct NoneCache {}

impl NoneCache {
    pub fn new() -> Self {
        NoneCache {}
    }
}

impl GenericCache for NoneCache {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn set(&mut self, _key: &str, _value: &str) {}
    fn cleanup(&mut self) {}
}
