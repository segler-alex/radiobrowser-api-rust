use crate::api::data::StationCheck;

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
impl From<StationCheckItem> for StationCheck {
    fn from(item: StationCheckItem) -> Self {
        StationCheck::new(
            item.station_uuid,
            item.check_uuid,
            item.source,
            item.codec,
            item.bitrate,
            if item.hls { 1 } else { 0 },
            if item.check_ok { 1 } else { 0 },
            item.check_time,
            item.url,
        )
    }
}
