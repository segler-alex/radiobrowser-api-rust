use chrono::NaiveDateTime;
use chrono::Utc;
use chrono::DateTime;
use chrono::SecondsFormat;
use crate::api::api_response::ApiResponse;
use crate::db::models::StationCheckItem;
use std::convert::TryFrom;
use std::error::Error;
use serde::{Serialize,Deserialize};

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

#[derive(PartialEq, Serialize, Deserialize)]
pub struct StationCheck {
    pub stationuuid: String,
    pub checkuuid: String,
    pub source: String,
    pub codec: String,
    pub bitrate: u32,
    pub hls: u8,
    pub ok: u8,
    pub timestamp_iso8601: Option<DateTime<Utc>>,
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
    pub do_not_index: Option<u8>,
    
    pub countrysubdivisioncode: Option<String>,
    pub server_software: Option<String>,
    pub sampling: Option<u32>,
    pub timing_ms: Option<u128>,
    pub languagecodes: Option<String>,
    pub ssl_error: Option<u8>,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
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
        timestamp_iso8601: Option<DateTime<Utc>>,
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
        do_not_index: Option<u8>,
        
        countrysubdivisioncode: Option<String>,
        server_software: Option<String>,
        sampling: Option<u32>,
        timing_ms: u128,
        languagecodes: Option<String>,
        ssl_error: u8,
        geo_lat: Option<f64>,
        geo_long: Option<f64>,
    ) -> Self {
        StationCheck {
            stationuuid,
            checkuuid,
            source,
            codec,
            bitrate,
            hls,
            ok,
            timestamp_iso8601,
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
            do_not_index,

            countrysubdivisioncode,
            server_software,
            sampling,
            timing_ms: Some(timing_ms),
            languagecodes,
            ssl_error: Some(ssl_error),
            geo_lat,
            geo_long,
        }
    }

    pub fn serialize_station_checks_csv(entries: Vec<StationCheck>) -> Result<String, Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        for entry in entries {
            wtr.serialize(entry)?;
        }
        
        wtr.flush()?;
        let x: Vec<u8> = wtr.into_inner()?;
        Ok(String::from_utf8(x).unwrap_or("encoding error".to_string()))
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
            if let Some(timestamp_iso8601) = entry.timestamp_iso8601 {
                xml.attr_esc("timestamp_iso8601", &timestamp_iso8601.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            }
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

            xml.attr_esc("countrysubdivisioncode", &entry.countrysubdivisioncode.unwrap_or_default())?;
            xml.attr_esc("server_software", &entry.server_software.unwrap_or_default())?;
            xml.attr_esc("sampling", &entry.sampling.unwrap_or(0).to_string())?;
            xml.attr_esc("timing_ms", &entry.timing_ms.unwrap_or(0).to_string())?;
            xml.attr_esc("languagecodes", &entry.languagecodes.unwrap_or_default())?;
            xml.attr_esc("ssl_error", &entry.ssl_error.unwrap_or(0).to_string())?;
            if let Some(geo_lat) = &entry.geo_lat {
                xml.attr_esc("geo_lat", &geo_lat.to_string())?;
            }
            if let Some(geo_long) = &entry.geo_long {
                xml.attr_esc("geo_long", &geo_long.to_string())?;
            }
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(list: Vec<StationCheck>, format: &str) -> Result<ApiResponse, Box<dyn Error>> {
        Ok(match format {
            "csv" => ApiResponse::Text(StationCheck::serialize_station_checks_csv(list)?),
            "json" => ApiResponse::Text(serde_json::to_string(&list)?),
            "xml" => ApiResponse::Text(StationCheck::serialize_station_checks(list)?),
            _ => ApiResponse::UnknownContentType,
        })
    }
}

impl TryFrom<StationCheckV0> for StationCheck {
    type Error = Box<dyn std::error::Error>;

    fn try_from(item: StationCheckV0) -> Result<Self, Self::Error> {
        let timestamp_iso8601 = NaiveDateTime::parse_from_str(&item.timestamp, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));

        Ok(StationCheck {
            stationuuid: item.stationuuid,
            checkuuid: item.checkuuid,
            source: item.source,
            codec: item.codec,
            bitrate: item.bitrate.parse()?,
            hls: item.hls.parse()?,
            ok: item.ok.parse()?,
            timestamp_iso8601,
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
            do_not_index: None,

            countrysubdivisioncode: None,
            server_software: None,
            sampling: None,
            timing_ms: None,
            languagecodes: None,
            ssl_error: None,
            geo_lat: None,
            geo_long: None,
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
            item.check_time_iso8601,
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
            item.do_not_index.map(|x| if x {1} else {0}),

            item.countrysubdivisioncode,
            item.server_software,
            item.sampling,
            item.timing_ms,
            item.languagecodes,
            if item.ssl_error { 1 } else { 0 },
            item.geo_lat,
            item.geo_long,
        )
    }
}