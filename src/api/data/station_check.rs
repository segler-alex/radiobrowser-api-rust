#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationCheckV0 {
    pub stationuuid: String,
    pub checkuuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: String,
    pub hls: String,
    pub ok: String,
    pub urlcache: String,
    pub timestamp: String,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationCheck {
    #[serde(skip_serializing)]
    pub id: i32,
    pub stationuuid: String,
    pub checkuuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: u8,
    pub ok: u8,
    pub timestamp: String,
    pub urlcache: String,
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
            xml.attr_esc("urlcache", &entry.urlcache)?;
            xml.attr_esc("timestamp", &entry.timestamp)?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(list: Vec<StationCheck>, format: &str) -> rouille::Response {
        match format {
            "json" => {
                let j = serde_json::to_string(&list).unwrap();
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
            },
            "xml" => {
                let j = StationCheck::serialize_station_checks(list).unwrap();
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
            },
            _ => rouille::Response::empty_406()
        }
    }
}


impl From<&StationCheckV0> for StationCheck {
    fn from(item: &StationCheckV0) -> Self {
        StationCheck {
            id: 0,
            stationuuid: item.stationuuid.clone(),
            checkuuid: item.checkuuid.clone(),
            source: item.source.clone(),
            codec: item.codec.clone(),
            bitrate: item.bitrate.parse().unwrap(),
            hls: item.hls.parse().unwrap(),
            ok: item.ok.parse().unwrap(),
            timestamp: item.timestamp.clone(),
            urlcache: item.urlcache.clone()
        }
    }
}