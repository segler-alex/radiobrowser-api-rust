#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    name: String,
    country: String,
    stationcount: u32,
}

impl State {
    pub fn new(name: String, country: String, stationcount: u32) -> Self {
        State {
            name,
            country,
            stationcount,
        }
    }

    pub fn serialize_state_list(entries: Vec<State>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("state")?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("country", &entry.country)?;
            xml.attr_esc("stationcount", &entry.stationcount.to_string())?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}