#[derive(Serialize, Deserialize)]
pub struct StationAddResult {
    ok: bool,
    message: String,
    id: u64,
    uuid: String,
    stream_check_ok: bool,
    stream_check_bitrate: u32,
    stream_check_codec: String,
    favicon_check_done: bool,
    favicon_check_ok: bool,
    favicon_check_url: String,
}

impl StationAddResult {
    pub fn new_ok(id: u64, stationuuid: String) -> StationAddResult {
        StationAddResult{
            ok: true,
            message: "added station successfully".to_string(),
            id: id,
            uuid: stationuuid,
            stream_check_ok: false,
            stream_check_bitrate: 0,
            stream_check_codec: "".to_string(),
            favicon_check_done: false,
            favicon_check_ok: false,
            favicon_check_url: "".to_string(),
        }
    }

    pub fn new_err(err: &str) -> StationAddResult {
        StationAddResult{
            ok: false,
            message: err.to_string(),
            id: 0,
            uuid: "".to_string(),
            stream_check_ok: false,
            stream_check_bitrate: 0,
            stream_check_codec: "".to_string(),
            favicon_check_done: false,
            favicon_check_ok: false,
            favicon_check_url: "".to_string(),
        }
    }

    pub fn serialize_xml(&self) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        xml.begin_elem("status")?;
        xml.attr_esc("ok", &self.ok.to_string())?;
        xml.attr_esc("message", &self.ok.to_string())?;
        xml.attr_esc("id", &self.id.to_string())?;
        xml.attr_esc("uuid", &self.uuid)?;
        xml.attr_esc("stream_check_ok", &self.stream_check_ok.to_string())?;
        xml.attr_esc("stream_check_bitrate", &self.stream_check_bitrate.to_string())?;
        xml.attr_esc("stream_check_codec", &self.stream_check_codec)?;
        xml.attr_esc("favicon_check_done", &self.favicon_check_done.to_string())?;
        xml.attr_esc("favicon_check_ok", &self.favicon_check_ok.to_string())?;
        xml.attr_esc("favicon_check_url", &self.favicon_check_url.to_string())?;
        xml.end_elem()?;
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(&self, format: &str) -> rouille::Response {
        match format {
            "json" => {
                let j = serde_json::to_string(&self).unwrap();
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
            },
            "xml" => {
                let j = self.serialize_xml().unwrap();
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
            },
            _ => rouille::Response::empty_406()
        }
    }
}