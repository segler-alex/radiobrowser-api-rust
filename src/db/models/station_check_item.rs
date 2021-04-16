use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct StationCheckItem {
    pub check_id: i32,
    pub check_time_iso8601: Option<DateTime<Utc>>,
    pub check_time: String,
    pub check_uuid: String,

    pub station_uuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: bool,
    pub check_ok: bool,
    pub url: String,

    pub metainfo_overrides_database: bool,
    pub public: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub countrycode: Option<String>,
    pub homepage: Option<String>,
    pub favicon: Option<String>,
    pub loadbalancer: Option<String>,
    pub do_not_index: Option<bool>,

    pub countrysubdivisioncode: Option<String>,
    pub server_software: Option<String>,
    pub sampling: Option<u32>,
    pub timing_ms: u128,
    pub languagecodes: Option<String>,
    pub ssl_error: bool,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
}
