use crate::api::api_response::ApiResponse;
use crate::db::models::StationCheckStepItem;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationCheckStep {
    pub stepuuid: String,
    pub parent_stepuuid: Option<String>,
    pub checkuuid: String,
    pub stationuuid: String,
    pub url: String,
    pub urltype: Option<String>,
    pub error: Option<String>,
    pub inserttime: DateTime<Utc>,
}

impl StationCheckStep {
    pub fn serialize_station_checks_csv(
        entries: Vec<StationCheckStep>,
    ) -> Result<String, Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        for entry in entries {
            wtr.serialize(entry)?;
        }
        wtr.flush()?;
        let x: Vec<u8> = wtr.into_inner()?;
        Ok(String::from_utf8(x).unwrap_or("encoding error".to_string()))
    }

    pub fn serialize_station_checks(entries: Vec<StationCheckStep>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("check")?;
            xml.attr_esc("stepuuid", &entry.stepuuid)?;
            if let Some(parent_stepuuid) = entry.parent_stepuuid {
                xml.attr_esc("parent_stepuuid", &parent_stepuuid)?;
            }
            xml.attr_esc("checkuuid", &entry.checkuuid)?;
            xml.attr_esc("stationuuid", &entry.stationuuid)?;
            xml.attr_esc("url", &entry.url)?;
            if let Some(urltype) = entry.urltype {
                xml.attr_esc("urltype", &urltype)?;
            }
            if let Some(error) = entry.error {
                xml.attr_esc("error", &error)?;
            }
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(
        list: Vec<StationCheckStep>,
        format: &str,
    ) -> Result<ApiResponse, Box<dyn Error>> {
        Ok(match format {
            "csv" => ApiResponse::Text(StationCheckStep::serialize_station_checks_csv(list)?),
            "json" => ApiResponse::Text(serde_json::to_string(&list)?),
            "xml" => ApiResponse::Text(StationCheckStep::serialize_station_checks(list)?),
            _ => ApiResponse::UnknownContentType,
        })
    }
}
impl From<StationCheckStepItem> for StationCheckStep {
    fn from(item: StationCheckStepItem) -> Self {
        StationCheckStep {
            stepuuid: item.stepuuid,
            parent_stepuuid: item.parent_stepuuid,
            checkuuid: item.checkuuid,
            stationuuid: item.stationuuid,
            url: item.url,
            urltype: item.urltype,
            error: item.error,
            inserttime: item.inserttime,
        }
    }
}
