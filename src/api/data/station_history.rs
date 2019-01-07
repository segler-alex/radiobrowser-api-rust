#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationHistoryV0 {
    id: String,
    changeuuid: String,
    stationuuid: String,
    name: String,
    url: String,
    homepage: String,
    favicon: String,
    tags: String,
    country: String,
    state: String,
    language: String,
    votes: String,
    negativevotes: String,
    lastchangetime: String,
    ip: String,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationHistoryCurrent {
    id: i32,
    changeuuid: String,
    stationuuid: String,
    pub name: String,
    pub url: String,
    homepage: String,
    favicon: String,
    tags: String,
    country: String,
    state: String,
    language: String,
    votes: i32,
    negativevotes: i32,
    lastchangetime: String,
    ip: String,
}

impl From<StationHistoryV0> for StationHistoryCurrent {
    fn from(item: StationHistoryV0) -> Self {
        StationHistoryCurrent {
            id: item.id.parse().unwrap(),
            changeuuid: item.changeuuid,
            stationuuid: item.stationuuid,
            name: item.name,
            url: item.url,
            homepage: item.homepage,
            favicon: item.favicon,
            tags: item.tags,
            country: item.country,
            state: item.state,
            language: item.language,
            votes: item.votes.parse().unwrap(),
            negativevotes: item.negativevotes.parse().unwrap(),
            lastchangetime: item.lastchangetime,
            ip: item.ip,
        }
    }
}

impl From<&StationHistoryV0> for StationHistoryCurrent {
    fn from(item: &StationHistoryV0) -> Self {
        StationHistoryCurrent {
            id: item.id.parse().unwrap(),
            changeuuid: item.changeuuid.clone(),
            stationuuid: item.stationuuid.clone(),
            name: item.name.clone(),
            url: item.url.clone(),
            homepage: item.homepage.clone(),
            favicon: item.favicon.clone(),
            tags: item.tags.clone(),
            country: item.country.clone(),
            state: item.state.clone(),
            language: item.language.clone(),
            votes: item.votes.parse().unwrap(),
            negativevotes: item.negativevotes.parse().unwrap(),
            lastchangetime: item.lastchangetime.clone(),
            ip: item.ip.clone(),
        }
    }
}

impl StationHistoryCurrent {
    pub fn new(
        id: i32,
        changeuuid: String,
        stationuuid: String,
        name: String,
        url: String,
        homepage: String,
        favicon: String,
        tags: String,
        country: String,
        state: String,
        language: String,
        votes: i32,
        negativevotes: i32,
        lastchangetime: String,
        ip: String,
    ) -> Self {
        StationHistoryCurrent {
            id,
            changeuuid,
            stationuuid,
            name,
            url,
            homepage,
            favicon,
            tags,
            country,
            state,
            language,
            votes,
            negativevotes,
            lastchangetime,
            ip,
        }
    }
    pub fn serialize_changes_list(entries: Vec<StationHistoryCurrent>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("station")?;
            let station_id_str = format!("{}", entry.id);
            xml.attr_esc("id", &station_id_str)?;
            xml.attr_esc("changeuuid", &entry.changeuuid)?;
            xml.attr_esc("stationuuid", &entry.stationuuid)?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("url", &entry.url)?;
            xml.attr_esc("homepage", &entry.homepage)?;
            xml.attr_esc("favicon", &entry.favicon)?;
            xml.attr_esc("tags", &entry.tags)?;
            xml.attr_esc("country", &entry.country)?;
            xml.attr_esc("state", &entry.state)?;
            xml.attr_esc("language", &entry.language)?;
            let station_votes_str = format!("{}", entry.votes);
            xml.attr_esc("votes", &station_votes_str)?;
            let station_negativevotes_str = format!("{}", entry.negativevotes);
            xml.attr_esc("negativevotes", &station_negativevotes_str)?;
            let station_lastchangetime_str = format!("{}", entry.lastchangetime);
            xml.attr_esc("lastchangetime", &station_lastchangetime_str)?;
            xml.attr_esc("ip", &entry.ip)?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }
}
