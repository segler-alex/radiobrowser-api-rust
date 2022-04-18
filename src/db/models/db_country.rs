use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct DBCountry {
    pub iso_3166_1: String,
    pub stationcount: u32,
}

impl DBCountry {
    pub fn new(iso_3166_1: String, stationcount: u32) -> Self {
        DBCountry {
            iso_3166_1,
            stationcount,
        }
    }
}
