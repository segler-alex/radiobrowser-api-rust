#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtraInfo {
    name: String,
    stationcount: u32,
}

impl ExtraInfo {
    pub fn new(name: String, stationcount:u32) -> Self {
        return ExtraInfo{
            name,
            stationcount,
        };
    }

    pub fn serialize_extra_list(entries: Vec<ExtraInfo>, tag_name: &str) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem(tag_name)?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("stationcount", &entry.stationcount.to_string())?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}