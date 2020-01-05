#[derive(Clone, Debug)]
pub struct StationCheckItem {
    pub check_id: i32,
    pub check_time: String,
    pub check_uuid: String,

    pub station_uuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: bool,
    pub check_ok: bool,
    pub url: String,
}
