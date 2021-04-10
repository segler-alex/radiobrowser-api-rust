use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ResultMessage {
    ok: bool,
    message: String,
}

impl ResultMessage {
    pub fn new(ok: bool, message: String) -> Self {
        ResultMessage{
            ok,
            message
        }
    }

    pub fn serialize_xml(&self) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
            xml.begin_elem("status")?;
                xml.attr_esc("ok", &self.ok.to_string())?;
                xml.attr_esc("message", &self.message)?;
            xml.end_elem()?;
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}