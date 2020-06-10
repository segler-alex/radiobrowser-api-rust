pub trait GenericCache {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
    fn cleanup(&mut self);
}
