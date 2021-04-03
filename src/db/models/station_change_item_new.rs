#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct StationChangeItemNew {
    pub name: String,
    pub url: String,
    pub homepage: String,
    pub favicon: String,
    pub country: String,
    pub state: String,
    pub countrycode: String,
    pub language: String,
    pub tags: String,
    pub votes: i32,

    pub changeuuid: String,
    pub stationuuid: String,
}