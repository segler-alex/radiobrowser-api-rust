use std::time::{Instant};
pub struct UuidWithTime {
    pub uuid: String,
    pub instant: Instant,
}

impl UuidWithTime {
    pub fn new(uuid: &str) -> Self{
        UuidWithTime {
            uuid: uuid.to_string(),
            instant: Instant::now(),
        }
    }
}