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
    pub clicktimestamp: String,
}

impl StationClick {
    pub fn new(
        stationuuid: String,
        clickuuid: String,
        clicktimestamp: String,
    ) -> Self {
        StationClick {
            stationuuid,
            clickuuid,
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
        Ok(StationClick {
            stationuuid: item.stationuuid,
            clickuuid: item.clickuuid,
            clicktimestamp: item.clicktimestamp,
        })
    }
}

impl From<StationClickItem> for StationClick {
    fn from(item: StationClickItem) -> Self {
        StationClick::new(
            item.stationuuid,
            item.clickuuid,
            item.clicktimestamp,
        )
    }
}