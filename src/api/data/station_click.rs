use chrono::NaiveDateTime;
use chrono::DateTime;
use chrono::Utc;
use chrono::SecondsFormat;
use crate::api::api_response::ApiResponse;
use crate::db::models::StationClickItem;
use std::convert::TryFrom;
use std::error::Error;
use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationClickV0 {
    pub stationuuid: String,
    pub clickuuid: String,
    pub clicktimestamp: String,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationClick {
    pub stationuuid: String,
    pub clickuuid: String,
    pub clicktimestamp_iso8601: Option<DateTime<Utc>>,
    pub clicktimestamp: String,
}

impl StationClick {
    pub fn new(
        stationuuid: String,
        clickuuid: String,
        clicktimestamp_iso8601: Option<DateTime<Utc>>,
        clicktimestamp: String,
    ) -> Self {
        StationClick {
            stationuuid,
            clickuuid,
            clicktimestamp_iso8601,
            clicktimestamp,
        }
    }

    pub fn serialize_station_clicks_csv(entries: Vec<StationClick>) -> Result<String, Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        for entry in entries {
            wtr.serialize(entry)?;
        }
        
        wtr.flush()?;
        let x: Vec<u8> = wtr.into_inner()?;
        Ok(String::from_utf8(x).unwrap_or("encoding error".to_string()))
    }

    pub fn serialize_station_clicks(entries: Vec<StationClick>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("click")?;
            xml.attr_esc("stationuuid", &entry.stationuuid)?;
            xml.attr_esc("clickuuid", &entry.clickuuid)?;
            if let Some(clicktimestamp_iso8601) = entry.clicktimestamp_iso8601 {
                xml.attr_esc("clicktimestamp_iso8601", &clicktimestamp_iso8601.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            }
            xml.attr_esc("clicktimestamp", &entry.clicktimestamp)?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(list: Vec<StationClick>, format: &str) -> Result<ApiResponse, Box<dyn Error>> {
        Ok(match format {
            "csv" => ApiResponse::Text(StationClick::serialize_station_clicks_csv(list)?),
            "json" => ApiResponse::Text(serde_json::to_string(&list)?),
            "xml" => ApiResponse::Text(StationClick::serialize_station_clicks(list)?),
            _ => ApiResponse::UnknownContentType,
        })
    }
}

impl TryFrom<StationClickV0> for StationClick {
    type Error = Box<dyn std::error::Error>;

    fn try_from(item: StationClickV0) -> Result<Self, Self::Error> {
        let clicktimestamp_iso8601 = NaiveDateTime::parse_from_str(&item.clicktimestamp, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));

        Ok(StationClick {
            stationuuid: item.stationuuid,
            clickuuid: item.clickuuid,
            clicktimestamp_iso8601,
            clicktimestamp: item.clicktimestamp,
        })
    }
}

impl From<StationClickItem> for StationClick {
    fn from(item: StationClickItem) -> Self {
        StationClick::new(
            item.stationuuid,
            item.clickuuid,
            item.clicktimestamp_iso8601,
            item.clicktimestamp,
        )
    }
}