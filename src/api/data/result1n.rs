#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Result1n {
    name: String,
    stationcount: u32,
}

impl Result1n {
    pub fn new(name: String, stationcount: u32) -> Self {
        Result1n {
            name,
            stationcount,
        }
    }
    pub fn serialize_result1n_list(type_str: &str, entries: Vec<Result1n>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem(type_str)?;
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