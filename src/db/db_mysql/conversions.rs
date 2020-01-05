use crate::db::models::StationItem;
use crate::db::models::StationCheckItem;
use crate::db::models::StationHistoryItem;
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
            id:                 row.take("StationID").unwrap(),
            changeuuid:         row.take("ChangeUuid").unwrap(),
            stationuuid:        row.take("StationUuid").unwrap_or("".to_string()),
            name:               row.take_opt("Name").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url:                row.take_opt("Url").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url_resolved:       row.take_opt("UrlCache").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            codec:              row.take_opt("Codec").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            bitrate:            row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
            hls:                row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0)==1,
            lastcheckok:        row.take_opt("LastCheckOK").unwrap_or(Ok(0)).unwrap_or(0)==1,
            favicon:            row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            tags:               row.take_opt("Tags").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            country:            row.take_opt("Country").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            countrycode:        row.take_opt("CountryCode").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            state:              row.take_opt("Language").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            language:           row.take_opt("Subcountry").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            votes:              row.take_opt("Votes").unwrap_or(Ok(0)).unwrap_or(0),
            lastchangetime:     row.take_opt("CreationFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            homepage:           row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            lastchecktime:      row.take_opt("LastCheckTimeFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            lastcheckoktime:    row.take_opt("LastCheckOkTimeFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            lastlocalchecktime: row.take_opt("LastLocalCheckTimeFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            clicktimestamp:     row.take_opt("ClickTimestampFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            clickcount:         row.take_opt("clickcount").unwrap_or(Ok(0)).unwrap_or(0),
            clicktrend:         row.take_opt("ClickTrend").unwrap_or(Ok(0)).unwrap_or(0),
        }
    }
}

impl From<Row> for StationHistoryItem {
    fn from(mut row: Row) -> Self {
        StationHistoryItem {
            id:                 row.take("StationChangeID").unwrap(),
            stationid:          row.take("StationID").unwrap(),
            changeuuid:         row.take("ChangeUuid").unwrap(),
            stationuuid:        row.take("StationUuid").unwrap(),
            name:               row.take_opt("Name").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url:                row.take_opt("Url").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            favicon:            row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            tags:               row.take_opt("Tags").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            country:            row.take_opt("Country").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            countrycode:        row.take_opt("CountryCode").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            state:              row.take_opt("Language").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            language:           row.take_opt("Subcountry").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            votes:              row.take_opt("Votes").unwrap_or(Ok(0)).unwrap_or(0),
            lastchangetime:     row.take_opt("CreationFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            homepage:           row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
        }
    }
}