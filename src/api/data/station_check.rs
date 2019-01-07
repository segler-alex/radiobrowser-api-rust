#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationCheck {
    id: i32,
    stationuuid: String,
    checkuuid: String,
    source: String,
    codec: String,
    bitrate: u32,
    hls: u8,
    ok: u8,
    timestamp: String,
    urlcache: String,
}

impl StationCheck {
    pub fn new(
        id: i32,
        stationuuid: String,
        checkuuid: String,
        source: String,
        codec: String,
        bitrate: u32,
        hls: u8,
        ok: u8,
        timestamp: String,
        urlcache: String,
    ) -> Self {
        StationCheck {
            id,
            stationuuid,
            checkuuid,
            source,
            codec,
            bitrate,
            hls,
            ok,
            timestamp,
            urlcache,
        }
    }

    pub fn serialize_station_checks(entries: Vec<StationCheck>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("check")?;
            xml.attr_esc("stationuuid", &entry.stationuuid)?;
            xml.attr_esc("checkuuid", &entry.checkuuid)?;
            xml.attr_esc("source", &entry.source)?;
            xml.attr_esc("codec", &entry.codec)?;
            xml.attr_esc("bitrate", &entry.bitrate.to_string())?;
            xml.attr_esc("hls", &entry.hls.to_string())?;
            xml.attr_esc("ok", &entry.ok.to_string())?;
            xml.attr_esc("timestamp", &entry.timestamp)?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}
