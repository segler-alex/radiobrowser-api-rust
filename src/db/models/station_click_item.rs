#[derive(Clone,Debug)]
pub struct StationClickItem {
    pub id: i32,
    pub stationuuid: String,
    pub ip: String,
    pub clickuuid: String,
    pub clicktimestamp: String,
}