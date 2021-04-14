use serde::{Serialize,Deserialize};

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct StationChangeItemNew {
    pub name: String,
    pub url: String,
    pub homepage: String,
    pub favicon: String,
    pub country: String,
    pub state: String,
    pub countrycode: String,
    pub language: String,
    pub languagecodes: String,
    pub tags: String,
    pub votes: i32,

    pub changeuuid: String,
    pub stationuuid: String,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
}