use crate::api::api_response::ApiResponse;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct StationAddResult {
    ok: bool,
    message: String,
    uuid: String,
}

impl StationAddResult {
    pub fn new_ok(stationuuid: String) -> StationAddResult {
        StationAddResult {
            ok: true,
            message: "added station successfully".to_string(),
            uuid: stationuuid,
        }
    }

    pub fn new_err(err: &str) -> StationAddResult {
        StationAddResult {
            ok: false,
            message: err.to_string(),
            uuid: "".to_string(),
        }
    }

    pub fn serialize_xml(&self) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        xml.begin_elem("status")?;
        xml.attr_esc("ok", &self.ok.to_string())?;
        xml.attr_esc("message", &self.ok.to_string())?;
        xml.attr_esc("uuid", &self.uuid)?;
        xml.end_elem()?;
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn from(result: Result<String, Box<dyn Error>>) -> StationAddResult {
        match result {
            Ok(res) => StationAddResult::new_ok(res),
            Err(err) => StationAddResult::new_err(&err.to_string()),
        }
    }

    pub fn get_response(&self, format: &str) -> Result<ApiResponse, Box<dyn Error>> {
        Ok(match format {
            "json" => ApiResponse::Text(serde_json::to_string(&self)?),
            "xml" => ApiResponse::Text(self.serialize_xml()?),
            _ => ApiResponse::UnknownContentType,
        })
    }
}
