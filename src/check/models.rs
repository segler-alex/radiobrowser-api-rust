#[derive(Clone,Debug)]
pub struct StationItem {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub url: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: bool,
    pub check_ok: bool,
    pub urlcache: String,
    pub favicon: String,
    pub homepage: String,
}

#[derive(Clone,Debug)]
pub struct StationCheckItem {
    pub check_id: i32,
    pub station_uuid: String,
    pub check_uuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: bool,
    pub check_ok: bool,
    pub check_time: String,
}

#[derive(Clone,Debug)]
pub struct StationCheckItemNew {
    pub station_uuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: bool,
    pub check_ok: bool,
    pub url: String,
}

#[derive(Clone,Debug)]
pub struct StationOldNew {
    pub old: StationItem,
    pub new: StationCheckItemNew,
}