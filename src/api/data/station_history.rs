use chrono::NaiveDateTime;
use crate::db::models::StationHistoryItem;
use celes::Country;
use std::error::Error;
use serde::{Serialize,Deserialize};
use chrono::DateTime;
use chrono::Utc;

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct StationHistoryV0 {
    changeuuid: String,
    stationuuid: String,
    name: String,
    url: String,
    homepage: String,
    favicon: String,
    tags: String,
    country: String,
    countrycode: String,
    state: String,
    language: String,
    votes: String,
    lastchangetime: String,
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct StationHistoryCurrent {
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
    pub languagecodes: Option<String>,
    pub votes: i32,
    pub lastchangetime: String,
    pub lastchangetime_iso8601: Option<DateTime<Utc>>,
    pub geo_lat: Option<f64>,
    pub geo_long: Option<f64>,
}

impl From<StationHistoryV0> for StationHistoryCurrent {
    fn from(item: StationHistoryV0) -> Self {
        let lastchangetime_iso8601 = NaiveDateTime::parse_from_str(&item.lastchangetime, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));

        StationHistoryCurrent {
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
            votes: item.votes.parse().unwrap(),
            lastchangetime: item.lastchangetime,
            lastchangetime_iso8601,
            geo_lat: None,
            geo_long: None,
        }
    }
}

impl From<&StationHistoryV0> for StationHistoryCurrent {
    fn from(item: &StationHistoryV0) -> Self {
        let lastchangetime_iso8601 = NaiveDateTime::parse_from_str(&item.lastchangetime, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|x|chrono::DateTime::<chrono::Utc>::from_utc(x, chrono::Utc));

        StationHistoryCurrent {
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
            languagecodes: None,
            votes: item.votes.parse().unwrap(),
            lastchangetime: item.lastchangetime.clone(),
            lastchangetime_iso8601,
            geo_lat: None,
            geo_long: None,
        }
    }
}

impl StationHistoryCurrent {
    pub fn serialize_changes_list_csv(entries: Vec<StationHistoryCurrent>) -> Result<String, Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        for entry in entries {
            wtr.serialize(entry)?;
        }
        
        wtr.flush()?;
        let x: Vec<u8> = wtr.into_inner()?;
        Ok(String::from_utf8(x).unwrap_or("encoding error".to_string()))
    }

    pub fn serialize_changes_list(entries: Vec<StationHistoryCurrent>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("station")?;
            xml.attr_esc("changeuuid", &entry.changeuuid)?;
            xml.attr_esc("stationuuid", &entry.stationuuid)?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("url", &entry.url)?;
            xml.attr_esc("homepage", &entry.homepage)?;
            xml.attr_esc("favicon", &entry.favicon)?;
            xml.attr_esc("tags", &entry.tags)?;
            xml.attr_esc("country", &entry.country)?;
            xml.attr_esc("countrycode", &entry.countrycode)?;
            xml.attr_esc("state", &entry.state)?;
            xml.attr_esc("language", &entry.language)?;
            let station_votes_str = format!("{}", entry.votes);
            xml.attr_esc("votes", &station_votes_str)?;
            let station_lastchangetime_str = format!("{}", entry.lastchangetime);
            xml.attr_esc("lastchangetime", &station_lastchangetime_str)?;
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
}

impl From<StationHistoryItem> for StationHistoryCurrent {
    fn from(item: StationHistoryItem) -> Self {
        StationHistoryCurrent {
            changeuuid: item.changeuuid,
            stationuuid: item.stationuuid,
            name: item.name,
            url: item.url,
            homepage: item.homepage,
            favicon: item.favicon,
            tags: item.tags,
            country: String::from(Country::from_alpha2(&item.countrycode).map(|c| c.long_name).unwrap_or("")),
            countrycode: item.countrycode,
            state: item.state,
            language: item.language,
            languagecodes: Some(item.languagecodes),
            votes: item.votes,
            lastchangetime: item.lastchangetime,
            lastchangetime_iso8601: item.lastchangetime_iso8601,
            geo_lat: item.geo_lat,
            geo_long: item.geo_long,
        }
    }
}