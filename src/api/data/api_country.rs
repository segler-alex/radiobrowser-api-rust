use crate::db::models::DBCountry;
use crate::api::api_response::ApiResponse;
use serde::{Deserialize, Serialize};
use std::error::Error;
use celes::Country;

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiCountry {
    pub name: String,
    pub iso_3166_1: String,
    pub stationcount: u32,
}

impl ApiCountry {
    /*
    pub fn new(name: String, iso_3166_1: String, stationcount: u32) -> Self {
        ApiCountry {
            name,
            iso_3166_1,
            stationcount,
        }
    }
    */

    pub fn new_with_code(iso_3166_1: String, stationcount: u32) -> Self {
        let name = Country::from_alpha2(&iso_3166_1).map(|d| d.long_name).unwrap_or("");
        ApiCountry {
            name: name.to_string(),
            iso_3166_1,
            stationcount,
        }
    }

    pub fn get_response<I, P>(list: I, format: &str) -> Result<ApiResponse, Box<dyn Error>>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiCountry>,
    {
        let list = list.into_iter();
        Ok(match format {
            "csv" => ApiResponse::Text(ApiCountry::serialize_to_csv(list)?),
            "json" => ApiResponse::Text(ApiCountry::serialize_to_json(list)?),
            "xml" => ApiResponse::Text(ApiCountry::serialize_to_xml(list)?),
            _ => ApiResponse::UnknownContentType,
        })
    }

    fn serialize_to_json<I, P>(entries: I) -> Result<String, Box<dyn Error>>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiCountry>,
    {
        let list: Vec<ApiCountry> = entries.into_iter().map(|item| item.into()).collect();
        Ok(serde_json::to_string(&list)?)
    }

    fn serialize_to_csv<I, P>(entries: I) -> Result<String, Box<dyn Error>>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiCountry>,
    {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        for entry in entries {
            let p: ApiCountry = entry.into();
            wtr.serialize(p)?;
        }
        wtr.flush()?;
        let x: Vec<u8> = wtr.into_inner()?;
        Ok(String::from_utf8(x).unwrap_or("encoding error".to_string()))
    }

    fn serialize_to_xml<I, P>(entries: I) -> std::io::Result<String>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiCountry>,
    {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            let entry: ApiCountry = entry.into();
            xml.begin_elem("language")?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("iso_3166_1", &entry.iso_3166_1)?;
            xml.attr_esc("stationcount", &entry.stationcount.to_string())?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}

impl From<DBCountry> for ApiCountry {
    fn from(item: DBCountry) -> Self {
        ApiCountry::new_with_code(item.iso_3166_1, item.stationcount)
    }
}