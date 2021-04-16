use crate::api::api_response::ApiResponse;
use crate::api::data::StationHistoryCurrent;
use crate::db::models::StationItem;
use std::error::Error;
use chrono::NaiveDateTime;
use chrono::DateTime;
use chrono::Utc;
use chrono::SecondsFormat;
use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationCachedInfo {
    ok: bool,
    message: String,
    stationuuid: String,
    name: String,
    url: String,
}

impl StationCachedInfo {
    pub fn serialize_cached_info(station: StationCachedInfo) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        xml.begin_elem("status")?;
        xml.attr_esc("ok", &station.ok.to_string())?;
        xml.attr_esc("message", &station.message)?;
        xml.attr_esc("stationuuid", &station.stationuuid)?;
        xml.attr_esc("name", &station.name)?;
        xml.attr_esc("url", &station.url)?;
        xml.end_elem()?;
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct StationV0 {
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub country: String,
    pub countrycode: String,
    pub state: String,
    pub language: String,
    pub votes: String,
    pub lastchangetime: String,
    pub codec: String,
    pub bitrate: String,
    pub hls: String,
    pub lastcheckok: String,
    pub lastchecktime: String,
    pub lastcheckoktime: String,
    pub clicktimestamp: String,
    pub clickcount: String,
    pub clicktrend: String,
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct Station {
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    pub url_resolved: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub country: String,
    pub countrycode: String,
    pub state: String,
    pub language: String,
    pub languagecodes: Option<String>,
    pub votes: i32,
    pub lastchangetime: String,
    pub lastchangetime_iso8601: Option<DateTime<Utc>>,
    pub codec: String,
    pub bitrate: u32,
    pub hls: i8,
    pub lastcheckok: i8,
    pub lastchecktime: String,
    pub lastchecktime_iso8601: Option<DateTime<Utc>>,
    pub lastcheckoktime: String,
    pub lastcheckoktime_iso8601: Option<DateTime<Utc>>,
    pub lastlocalchecktime: String,
    pub lastlocalchecktime_iso8601: Option<DateTime<Utc>>,
    pub clicktimestamp: String,
    pub clicktimestamp_iso8601: Option<DateTime<Utc>>,
    pub clickcount: u32,
    pub clicktrend: i32,
    pub ssl_error: Option<u8>,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
}

impl Station {
    pub fn extract_cached_info(station: Station, message: &str) -> StationCachedInfo {
        return StationCachedInfo {
            ok: station.lastcheckok == 1,
            message: message.to_string(),
            stationuuid: station.stationuuid,
            name: station.name,
            url: station.url_resolved,
        };
    }

    pub fn serialize_to_csv(entries: Vec<Station>) -> Result<String, Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        for entry in entries {
            wtr.serialize(entry)?;
        }
        
        wtr.flush()?;
        let x: Vec<u8> = wtr.into_inner()?;
        Ok(String::from_utf8(x).unwrap_or("encoding error".to_string()))
    }

    pub fn serialize_station_list(entries: Vec<Station>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("station")?;
            xml.attr_esc("changeuuid", &entry.changeuuid)?;
            xml.attr_esc("stationuuid", &entry.stationuuid)?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("url", &entry.url)?;
            xml.attr_esc("url_resolved", &entry.url_resolved)?;
            xml.attr_esc("homepage", &entry.homepage)?;
            xml.attr_esc("favicon", &entry.favicon)?;
            xml.attr_esc("tags", &entry.tags)?;
            xml.attr_esc("country", &entry.country)?;
            xml.attr_esc("countrycode", &entry.countrycode)?;
            xml.attr_esc("state", &entry.state)?;
            xml.attr_esc("language", &entry.language)?;
            if let Some(languagecodes) = entry.languagecodes {
                xml.attr_esc("languagecodes", &languagecodes)?;
            }
            let station_votes_str = format!("{}", entry.votes);
            xml.attr_esc("votes", &station_votes_str)?;
            xml.attr_esc("lastchangetime", &entry.lastchangetime)?;
            if let Some(lastchangetime_iso8601) = &entry.lastchangetime_iso8601 {
                xml.attr_esc("lastchangetime_iso8601", &lastchangetime_iso8601.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            }
            xml.attr_esc("codec", &entry.codec)?;
            let station_bitrate = format!("{}", entry.bitrate);
            xml.attr_esc("bitrate", &station_bitrate)?;
            let station_hls = format!("{}", entry.hls);
            xml.attr_esc("hls", &station_hls)?;
            let station_lastcheckok = format!("{}", entry.lastcheckok);
            xml.attr_esc("lastcheckok", &station_lastcheckok)?;
            xml.attr_esc("lastchecktime", &entry.lastchecktime)?;
            if let Some(lastchecktime_iso8601) = &entry.lastchecktime_iso8601 {
                xml.attr_esc("lastchecktime_iso8601", &lastchecktime_iso8601.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            }
            xml.attr_esc("lastcheckoktime", &entry.lastcheckoktime)?;
            if let Some(lastcheckoktime_iso8601) = &entry.lastcheckoktime_iso8601 {
                xml.attr_esc("lastcheckoktime_iso8601", &lastcheckoktime_iso8601.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            }
            xml.attr_esc("lastlocalchecktime", &entry.lastlocalchecktime)?;
            if let Some(lastlocalchecktime_iso8601) = &entry.lastlocalchecktime_iso8601 {
                xml.attr_esc("lastlocalchecktime_iso8601", &lastlocalchecktime_iso8601.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            }
            xml.attr_esc("clicktimestamp", &entry.clicktimestamp)?;
            if let Some(clicktimestamp_iso8601) = &entry.clicktimestamp_iso8601 {
                xml.attr_esc("clicktimestamp_iso8601", &clicktimestamp_iso8601.to_rfc3339_opts(SecondsFormat::Secs, true))?;
            }
            let station_clickcount = format!("{}", entry.clickcount);
            xml.attr_esc("clickcount", &station_clickcount)?;
            let station_clicktrend = format!("{}", entry.clicktrend);
            xml.attr_esc("clicktrend", &station_clicktrend)?;
            if let Some(ssl_error) = entry.ssl_error {
                let station_ssl_error = format!("{}", ssl_error);
                xml.attr_esc("ssl_error", &station_ssl_error)?;
            }
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

    pub fn serialize_to_m3u(list: Vec<Station>, use_cached_url: bool) -> String {
        let mut j = String::with_capacity(200 * list.len());
        j.push_str("#EXTM3U\r\n");
        for item in list {
            j.push_str("#RADIOBROWSERUUID:");
            j.push_str(&item.stationuuid);
            j.push_str("\r\n");
            j.push_str("#EXTINF:1,");
            j.push_str(&item.name);
            j.push_str("\r\n");
            if use_cached_url {
                j.push_str(&item.url_resolved);
            } else {
                j.push_str(&item.url);
            }
            j.push_str("\r\n\r\n");
        }
        j
    }

    pub fn serialize_to_pls(list: Vec<Station>, use_cached_url: bool) -> String {
        let mut j = String::with_capacity(200 * list.len());
        j.push_str("[playlist]\r\n");
        let mut i = 1;
        j.push_str(&format!("NumberOfEntries={}\r\n", list.len()));
        for item in list {
            let i_str = i.to_string();
            j.push_str("Title");
            j.push_str(&i_str);
            j.push_str("=");
            j.push_str(&item.name);
            j.push_str("\r\n");
            j.push_str("File");
            j.push_str(&i_str);
            j.push_str("=");
            if use_cached_url {
                j.push_str(&item.url_resolved);
            } else {
                j.push_str(&item.url);
            }
            j.push_str("\r\n\r\n");
            i += 1;
        }
        j
    }

    pub fn serialize_to_xspf(entries: Vec<Station>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.dtd("UTF-8")?;
        xml.begin_elem("playlist")?;
        xml.attr_esc("version", "1")?;
        xml.attr_esc("xmlns", "http://xspf.org/ns/0/")?;
        xml.begin_elem("trackList")?;
        for entry in entries {
            xml.begin_elem("track")?;
            xml.elem_text("title", &entry.name)?;
            xml.elem_text("location", &entry.url)?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    // Syntax checked with http://ttl.summerofcode.be/
    fn serialize_to_ttl_single(&self) -> String {
        format!(
            r#"<http://radio-browser.info/radio/{stationuuid}>
    rdf:type schema:RadioStation ;
    dcterms:identifier "{stationuuid}" ;
    schema:PropertyValue [
        schema:name "changeuuid" ;
        schema:value "{changeuuid}"
    ] ;
    schema:PropertyValue [
        schema:name "stationuuid" ;
        schema:value "{stationuuid}"
    ] ;
    schema:name "{name}" ;
    schema:url <{url}> ;
    schema:sameAs <{homepage}> ;
    schema:logo <{favicon}> ;
    schema:Country [
        schema:name "{country}" ;
    ] ;
    schema:CountryCode [
        schema:name "{countrycode}" ;
    ] ;
    schema:State [
        schema:name "{state}" ;
    ] ;
    schema:Language [
        schema:name "{language}" ;
    ] ;
    schema:PropertyValue [
        schema:name "votes" ;
        schema:value "{votes}"
    ] ;
    schema:PropertyValue [
        schema:name "lastchangetime" ;
        schema:value "{lastchangetime}"
    ] ;
    schema:PropertyValue [
        schema:name "codec" ;
        schema:value "{codec}"
    ] ;
    schema:PropertyValue [
        schema:name "bitrate" ;
        schema:value "{bitrate}"
    ] ;
    schema:PropertyValue [
        schema:name "hls" ;
        schema:value "{hls}"
    ] ;
    schema:PropertyValue [
        schema:name "lastcheckok" ;
        schema:value "{lastcheckok}"
    ] ;
    schema:PropertyValue [
        schema:name "lastchecktime" ;
        schema:value "{lastchecktime}"
    ] ;
    schema:PropertyValue [
        schema:name "lastcheckoktime" ;
        schema:value "{lastcheckoktime}"
    ] ;
    schema:PropertyValue [
        schema:name "clicktimestamp" ;
        schema:value "{clicktimestamp}"
    ] ;
    schema:PropertyValue [
        schema:name "clickcount" ;
        schema:value "{clickcount}"
    ] ;
    schema:PropertyValue [
        schema:name "clicktrend" ;
        schema:value "{clicktrend}"
    ] ;
    .{newline}"#,
            stationuuid = self.stationuuid,
            changeuuid = self.changeuuid,
            name = self.name,
            url = self.url,
            lastchangetime = self.lastchecktime,
            lastchecktime = self.lastchecktime,
            lastcheckoktime = self.lastcheckoktime,
            clicktimestamp = self.clicktimestamp,
            homepage = self.homepage,
            favicon = self.favicon,
            country = self.country,
            countrycode = self.countrycode,
            state = self.state,
            language = self.language,
            votes = self.votes,
            codec = self.codec,
            bitrate = self.bitrate,
            hls = self.hls,
            lastcheckok = self.lastcheckok,
            clickcount = self.clickcount,
            clicktrend = self.clicktrend,
            newline = "\r\n\r\n"
        )
    }

    pub fn serialize_to_ttl(list: Vec<Station>) -> String {
        let mut j = String::with_capacity(200 * list.len());

        j.push_str(
            r#"@prefix dcterms: <http://purl.org/dc/terms/> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix schema: <http://schema.org/> .
    @prefix wdrs: <https://www.w3.org/2007/05/powder-s#> .
    "#,
        );

        for entry in list {
            let x = entry.serialize_to_ttl_single();
            j.push_str(&x);
        }

        j
    }

    pub fn get_response(list: Vec<Station>, format: &str) -> Result<ApiResponse, Box<dyn Error>> {
        Ok(match format {
            "csv" => ApiResponse::Text(Station::serialize_to_csv(list)?),
            "json" => ApiResponse::Text(serde_json::to_string(&list)?),
            "xml" => ApiResponse::Text(Station::serialize_station_list(list)?),
            "m3u" => ApiResponse::Text(Station::serialize_to_m3u(list, false)),
            "pls" => ApiResponse::Text(Station::serialize_to_pls(list, false)),
            "xspf" => ApiResponse::Text(Station::serialize_to_xspf(list)?),
            "ttl" => ApiResponse::Text(Station::serialize_to_ttl(list)),
            _ => ApiResponse::UnknownContentType,
        })
    }
}

impl From<&StationHistoryCurrent> for Station {
    fn from(item: &StationHistoryCurrent) -> Self {
        let lastchangetime_iso8601 = NaiveDateTime::parse_from_str(&item.lastchangetime, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));
        
        Station {
            changeuuid: item.changeuuid.clone(),
            stationuuid: item.stationuuid.clone(),
            name: item.name.clone(),
            url: item.url.clone(),
            homepage: item.homepage.clone(),
            favicon: item.favicon.clone(),
            tags: item.tags.clone(),
            country: item.country.clone(),
            countrycode: item.countrycode.clone(),
            state: item.state.clone(),
            language: item.language.clone(),
            languagecodes: item.languagecodes.clone(),
            votes: item.votes,
            lastchangetime: item.lastchangetime.clone(),
            lastchangetime_iso8601: lastchangetime_iso8601,
            bitrate: 0,
            clickcount: 0,
            clicktimestamp: String::from(""),
            clicktimestamp_iso8601: None,
            clicktrend: 0,
            codec: String::from(""),
            hls: 0,
            lastcheckok: 0,
            lastcheckoktime: String::from(""),
            lastcheckoktime_iso8601: None,
            lastchecktime: String::from(""),
            lastchecktime_iso8601: None,
            lastlocalchecktime: String::from(""),
            lastlocalchecktime_iso8601: None,
            url_resolved: String::from(""),
            ssl_error: None,
            geo_lat: item.geo_lat,
            geo_long: item.geo_long,
        }
    }
}

impl From<StationItem> for Station {
    fn from(item: StationItem) -> Self {
        Station {
            changeuuid: item.changeuuid,
            stationuuid: item.stationuuid,
            name: item.name,
            url: item.url,
            homepage: item.homepage,
            favicon: item.favicon,
            tags: item.tags,
            country: item.country,
            countrycode: item.countrycode,
            state: item.state,
            language: item.language,
            languagecodes: Some(item.languagecodes),
            votes: item.votes,
            lastchangetime: item.lastchangetime,
            lastchangetime_iso8601: item.lastchangetime_iso8601,
            bitrate: item.bitrate,
            clickcount: item.clickcount,
            clicktimestamp: item.clicktimestamp,
            clicktimestamp_iso8601: item.clicktimestamp_iso8601,
            clicktrend: item.clicktrend,
            codec: item.codec,
            hls: if item.hls { 1 } else { 0 },
            lastcheckok: if item.lastcheckok { 1 } else { 0 },
            lastcheckoktime: item.lastcheckoktime,
            lastcheckoktime_iso8601: item.lastcheckoktime_iso8601,
            lastchecktime: item.lastchecktime,
            lastchecktime_iso8601: item.lastchecktime_iso8601,
            lastlocalchecktime: item.lastlocalchecktime,
            lastlocalchecktime_iso8601: item.lastlocalchecktime_iso8601,
            url_resolved: item.url_resolved,
            ssl_error: Some(if item.ssl_error { 1 } else { 0 }),
            geo_lat: item.geo_lat,
            geo_long: item.geo_long,
        }
    }
}

impl From<StationV0> for Station {
    fn from(item: StationV0) -> Self {
        let lastchangetime_iso8601 = NaiveDateTime::parse_from_str(&item.lastchangetime, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));
        let clicktimestamp_iso8601 = NaiveDateTime::parse_from_str(&item.clicktimestamp, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));
        let lastcheckoktime_iso8601 = NaiveDateTime::parse_from_str(&item.lastcheckoktime, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));
        let lastchecktime_iso8601 = NaiveDateTime::parse_from_str(&item.lastchecktime, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));

        Station {
            changeuuid: item.changeuuid,
            stationuuid: item.stationuuid,
            name: item.name,
            url: item.url,
            homepage: item.homepage,
            favicon: item.favicon,
            tags: item.tags,
            country: item.country,
            countrycode: item.countrycode,
            state: item.state,
            language: item.language,
            languagecodes: None,
            votes: item.votes.parse().unwrap_or(0),
            lastchangetime: item.lastchangetime,
            lastchangetime_iso8601: lastchangetime_iso8601,
            bitrate: item.bitrate.parse().unwrap_or(0),
            clickcount: item.clickcount.parse().unwrap_or(0),
            clicktimestamp: item.clicktimestamp,
            clicktimestamp_iso8601: clicktimestamp_iso8601,
            clicktrend: item.clicktrend.parse().unwrap_or(0),
            codec: item.codec,
            hls: item.hls.parse().unwrap_or(0),
            lastcheckok: item.lastcheckok.parse().unwrap_or(0),
            lastcheckoktime: item.lastcheckoktime,
            lastcheckoktime_iso8601: lastcheckoktime_iso8601,
            lastchecktime: item.lastchecktime,
            lastchecktime_iso8601: lastchecktime_iso8601,
            lastlocalchecktime: String::from(""),
            lastlocalchecktime_iso8601: None,
            url_resolved: String::from(""),
            ssl_error: None,
            geo_lat: None,
            geo_long: None,
        }
    }
}
