use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct StationClickItem {
    pub id: i32,
    pub stationuuid: String,
    pub ip: String,
    pub clickuuid: String,
    pub clicktimestamp_iso8601: Option<DateTime<Utc>>,
    pub clicktimestamp: String,
}
