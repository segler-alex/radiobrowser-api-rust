use crate::db::models::StationItem;
use crate::db::models::StationCheckItem;
use mysql;
use mysql::Row;

impl From<Row> for StationCheckItem {
    fn from(mut row: Row) -> Self {
        StationCheckItem {
            check_id:       row.take("CheckID").unwrap(),
            station_uuid:   row.take("StationUuid").unwrap_or("".to_string()),
            check_uuid:     row.take("CheckUuid").unwrap_or("".to_string()),
            source:         row.take("Source").unwrap_or("".to_string()),
            codec:          row.take_opt("Codec").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            bitrate:        row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
            hls:            row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0) == 1,
            check_ok:       row.take_opt("CheckOK").unwrap_or(Ok(0)).unwrap_or(0) == 1,
            check_time:     row.take_opt("CheckTimeFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url:            row.take_opt("UrlCache").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
        }
    }
}

impl From<Row> for StationItem {
    fn from(mut row: Row) -> Self {
        StationItem {
            id:              row.take("StationID").unwrap(),
            stationuuid:     row.take("StationUuid").unwrap_or("".to_string()),
            name:            row.take_opt("Name").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url:             row.take_opt("Url").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url_resolved:    row.take_opt("UrlCache").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            codec:           row.take_opt("Codec").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            bitrate:         row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
            hls:             row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0)==1,
            lastcheckok:     row.take_opt("LastCheckOK").unwrap_or(Ok(0)).unwrap_or(0)==1,
            favicon:         row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            homepage:        row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
        }
    }
}