#[derive(Clone,Debug)]
pub struct StationClickItemNew {
    pub stationid: i32,
    pub stationuuid: String,
    pub ip: String,
    pub clickuuid: String,
    pub clicktimestamp: String,
}