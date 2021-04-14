use serde::{Serialize,Deserialize};

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct StationHistoryItem {
    pub id: i32,
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub countrycode: String,
    pub state: String,
    pub language: String,
    pub languagecodes: String,
    pub votes: i32,
    pub lastchangetime: String,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
}
