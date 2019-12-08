#[derive(Serialize, Deserialize)]
pub struct Status {
    pub supported_version: u32,
    pub software_version: Option<String>,
    status: String,
    stations: u64,
    stations_broken: u64,
    tags: u64,
    clicks_last_hour: u64,
    clicks_last_day: u64,
    languages: u64,
    countries: u64,
}

impl Status{
    pub fn new(
        supported_version: u32,
        software_version: Option<String>,
        status: String,
        stations: u64,
        stations_broken: u64,
        tags: u64,
        clicks_last_hour: u64,
        clicks_last_day: u64,
        languages: u64,
        countries: u64
    ) -> Self {
        Status{
            supported_version,
            software_version,
            status,
            stations,
            stations_broken,
            tags,
            clicks_last_hour,
            clicks_last_day,
            languages,
            countries,
        }
    }

    pub fn serialize_xml(&self) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        {
            xml.begin_elem("stats")?;
            let s = self.status.clone();
                xml.attr_esc("supported_version", &self.supported_version.to_string())?;
                if let Some(software_version) = &self.software_version {
                    xml.attr_esc("software_version", &software_version)?;
                }
                xml.attr_esc("status", &s)?;
                xml.attr_esc("stations", &self.stations.to_string())?;
                xml.attr_esc("stations_broken", &self.stations_broken.to_string())?;
                xml.attr_esc("tags", &self.tags.to_string())?;
                xml.attr_esc("clicks_last_hour", &self.clicks_last_hour.to_string())?;
                xml.attr_esc("clicks_last_day", &self.clicks_last_day.to_string())?;
                xml.attr_esc("languages", &self.languages.to_string())?;
                xml.attr_esc("countries", &self.countries.to_string())?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap())
    }
}