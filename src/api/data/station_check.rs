use crate::db::models::StationCheckItem;
use std::convert::TryFrom;
use std::error::Error;

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
    pub stationuuid: String,
    pub checkuuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: u8,
    pub ok: u8,
    pub timestamp: String,
    pub urlcache: String,

    pub metainfo_overrides_database: Option<u8>,
    pub public: Option<u8>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub countrycode: Option<String>,
    pub homepage: Option<String>,
    pub favicon: Option<String>,
    pub loadbalancer: Option<String>,
}

impl StationCheck {
    pub fn new(
        stationuuid: String,
        checkuuid: String,
        source: String,
        codec: String,
        bitrate: u32,
        hls: u8,
        ok: u8,
        timestamp: String,
        urlcache: String,

        metainfo_overrides_database: Option<u8>,
        public: Option<u8>,
        name: Option<String>,
        description: Option<String>,
        tags: Option<String>,
        countrycode: Option<String>,
        homepage: Option<String>,
        favicon: Option<String>,
        loadbalancer: Option<String>,
    ) -> Self {
        StationCheck {
            stationuuid,
            checkuuid,
            source,
            codec,
            bitrate,
            hls,
            ok,
            timestamp,
            urlcache,

            metainfo_overrides_database,
            public,
            name,
            description,
            tags,
            countrycode,
            homepage,
            favicon,
            loadbalancer,
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

            xml.attr_esc("metainfo_overrides_database", &entry.metainfo_overrides_database.unwrap_or(0).to_string())?;
            xml.attr_esc("public", &entry.public.unwrap_or(0).to_string())?;
            xml.attr_esc("name", &entry.name.unwrap_or_default())?;
            xml.attr_esc("description", &entry.description.unwrap_or_default())?;
            xml.attr_esc("tags", &entry.tags.unwrap_or_default())?;
            xml.attr_esc("homepage", &entry.homepage.unwrap_or_default())?;
            xml.attr_esc("loadbalancer", &entry.loadbalancer.unwrap_or_default())?;
            xml.attr_esc("favicon", &entry.favicon.unwrap_or_default())?;
            xml.attr_esc("countrycode", &entry.countrycode.unwrap_or_default())?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(list: Vec<StationCheck>, format: &str) -> Result<rouille::Response, Box<dyn Error>> {
        Ok(match format {
            "json" => {
                let j = serde_json::to_string(&list)?;
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
            },
            "xml" => {
                let j = StationCheck::serialize_station_checks(list)?;
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
            },
            _ => rouille::Response::empty_406()
        })
    }
}

impl TryFrom<StationCheckV0> for StationCheck {
    type Error = Box<dyn std::error::Error>;

    fn try_from(item: StationCheckV0) -> Result<Self, Self::Error> {
        Ok(StationCheck {
            stationuuid: item.stationuuid,
            checkuuid: item.checkuuid,
            source: item.source,
            codec: item.codec,
            bitrate: item.bitrate.parse()?,
            hls: item.hls.parse()?,
            ok: item.ok.parse()?,
            timestamp: item.timestamp,
            urlcache: item.urlcache,
            
            metainfo_overrides_database: None,
            public: None,
            name: None,
            description: None,
            tags: None,
            countrycode: None,
            homepage: None,
            favicon: None,
            loadbalancer: None,
        })
    }
}

impl From<StationCheckItem> for StationCheck {
    fn from(item: StationCheckItem) -> Self {
        StationCheck::new(
            item.station_uuid,
            item.check_uuid,
            item.source,
            item.codec,
            item.bitrate,
            if item.hls { 1 } else { 0 },
            if item.check_ok { 1 } else { 0 },
            item.check_time,
            item.url,

            if item.metainfo_overrides_database {Some(1)} else {Some(0)},
            item.public.map(|x| if x {1} else {0}),
            item.name,
            item.description,
            item.tags,
            item.countrycode,
            item.homepage,
            item.favicon,
            item.loadbalancer,
        )
    }
}