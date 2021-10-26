use crate::api::api_response::ApiResponse;
use crate::config::convert_language_to_code;
use crate::db::models::ExtraInfo;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiLanguage {
    pub name: String,
    pub iso_639: Option<String>,
    pub stationcount: u32,
}

impl ApiLanguage {
    pub fn new(name: String, iso_639: Option<String>, stationcount: u32) -> Self {
        ApiLanguage {
            name,
            iso_639,
            stationcount,
        }
    }

    pub fn get_response<I, P>(list: I, format: &str) -> Result<ApiResponse, Box<dyn Error>>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiLanguage>,
    {
        let list = list.into_iter();
        Ok(match format {
            "csv" => ApiResponse::Text(ApiLanguage::serialize_to_csv(list)?),
            "json" => ApiResponse::Text(ApiLanguage::serialize_to_json(list)?),
            "xml" => ApiResponse::Text(ApiLanguage::serialize_to_xml(list)?),
            _ => ApiResponse::UnknownContentType,
        })
    }

    fn serialize_to_json<I, P>(entries: I) -> Result<String, Box<dyn Error>>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiLanguage>,
    {
        let list: Vec<ApiLanguage> = entries.into_iter().map(|item| item.into()).collect();
        Ok(serde_json::to_string(&list)?)
    }

    fn serialize_to_csv<I, P>(entries: I) -> Result<String, Box<dyn Error>>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiLanguage>,
    {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        for entry in entries {
            let p: ApiLanguage = entry.into();
            wtr.serialize(p)?;
        }
        wtr.flush()?;
        let x: Vec<u8> = wtr.into_inner()?;
        Ok(String::from_utf8(x).unwrap_or("encoding error".to_string()))
    }

    fn serialize_to_xml<I, P>(entries: I) -> std::io::Result<String>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiLanguage>,
    {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            let entry: ApiLanguage = entry.into();
            xml.begin_elem("language")?;
            xml.attr_esc("name", &entry.name)?;
            if let Some(iso_639) = entry.iso_639 {
                xml.attr_esc("iso_639", &iso_639)?;
            }
            xml.attr_esc("stationcount", &entry.stationcount.to_string())?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}

impl From<ExtraInfo> for ApiLanguage {
    fn from(item: ExtraInfo) -> Self {
        let codes = convert_language_to_code(&item.name);
        ApiLanguage::new(item.name, codes, item.stationcount)
    }
}
