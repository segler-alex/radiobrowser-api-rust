extern crate mysql;
extern crate xml_writer;
extern crate chrono;

use std::collections::HashMap;
use db::mysql::Value;
use db::mysql::QueryResult;
use std;
use thread;

pub struct Connection {
    pool: mysql::Pool
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Station {
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
    codec: String,
    bitrate: u32,
    hls: i8,
    lastcheckok: i8,
    lastchecktime: String,
    lastcheckoktime: String,
    clicktimestamp: String,
    clickcount: u32,
    clicktrend: i32
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationHistory {
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
    ip: String
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Result1n {
    name: String,
    value: String,
    stationcount: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    name: String,
    value: String,
    country: String,
    stationcount: u32,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtraInfo {
    name: String,
    value: String,
    stationcount: u32,
    stationcountworking: u32
}

pub fn serialize_to_m3u(list: Vec<Station>) -> String {
    let mut j = String::with_capacity(200 * list.len());
    j.push_str("#EXTM3U\r\n");
    for item in list {
        j.push_str("#EXTINF:1,");
        j.push_str(&item.name);
        j.push_str("\r\n");
        j.push_str(&item.url);
        j.push_str("\r\n\r\n");
    }
    j
}

pub fn serialize_to_pls(list: Vec<Station>) -> String {
    let mut j = String::with_capacity(200 * list.len());
    j.push_str("[playlist]\r\n");
    let mut i = 1;
    for item in list {
        let i_str = i.to_string();
        j.push_str("File");
        j.push_str(&i_str);
        j.push_str("=");
        j.push_str(&item.name);
        j.push_str("\r\n");
        j.push_str("Title");
        j.push_str(&i_str);
        j.push_str("=");
        j.push_str(&item.url);
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
    for entry in entries{
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

pub fn serialize_result1n_list(type_str: &str, entries: Vec<Result1n>) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    for entry in entries{
        xml.begin_elem(type_str)?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("value", &entry.value)?;
            let count_str = format!("{}", entry.stationcount);
            xml.attr_esc("stationcount", &count_str)?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
}

pub fn serialize_state_list(entries: Vec<State>) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    for entry in entries{
        xml.begin_elem("state")?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("value", &entry.value)?;
            xml.attr_esc("country", &entry.country)?;
            let count_str = format!("{}", entry.stationcount);
            xml.attr_esc("stationcount", &count_str)?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
}

pub fn serialize_extra_list(entries: Vec<ExtraInfo>, tag_name: &str) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    for entry in entries{
        xml.begin_elem(tag_name)?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("value", &entry.value)?;
            let count_str = format!("{}", entry.stationcount);
            xml.attr_esc("stationcount", &count_str)?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
}

pub fn serialize_station_list(entries: Vec<Station>) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    for entry in entries{
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
            xml.attr_esc("codec", &entry.codec)?;
            let station_bitrate = format!("{}", entry.bitrate);
            xml.attr_esc("bitrate", &station_bitrate)?;
            let station_hls = format!("{}", entry.hls);
            xml.attr_esc("hls", &station_hls)?;
            let station_lastcheckok = format!("{}", entry.lastcheckok);
            xml.attr_esc("lastcheckok", &station_lastcheckok)?;
            let station_lastchecktime_str = format!("{}", entry.lastchecktime);
            xml.attr_esc("lastchecktime", &station_lastchecktime_str)?;
            let station_lastcheckoktime_str = format!("{}", entry.lastcheckoktime);
            xml.attr_esc("lastcheckoktime", &station_lastcheckoktime_str)?;
            let station_clicktimestamp_str = format!("{}", entry.clicktimestamp);
            xml.attr_esc("clicktimestamp", &station_clicktimestamp_str)?;
            let station_clickcount = format!("{}", entry.clickcount);
            xml.attr_esc("clickcount", &station_clickcount)?;
            let station_clicktrend = format!("{}", entry.clicktrend);
            xml.attr_esc("clicktrend", &station_clicktrend)?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
}

pub fn serialize_changes_list(entries: Vec<StationHistory>) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    for entry in entries{
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

impl Connection {
    const COLUMNS: &'static str = "StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,
    Tags,Country,Subcountry,Language,Votes,NegativeVotes,
    Date_Format(Creation,'%Y-%m-%d %H:%i:%s') AS CreationFormated,
    Ip,Codec,Bitrate,Hls,LastCheckOK,
    LastCheckTime,
    Date_Format(LastCheckTime,'%Y-%m-%d %H:%i:%s') AS LastCheckTimeFormated,
    LastCheckOkTime,
    Date_Format(LastCheckOkTime,'%Y-%m-%d %H:%i:%s') AS LastCheckOkTimeFormated,
    ClickTimestamp,
    Date_Format(ClickTimestamp,'%Y-%m-%d %H:%i:%s') AS ClickTimestampFormated,
    clickcount,ClickTrend";

    pub fn get_stations_by_all(&self) -> Vec<Station> {
        let query : String;
        query = format!("SELECT {columns} from Station ORDER BY Name", columns = Connection::COLUMNS);
        self.get_stations_query(query)
    }

    pub fn filter_order(&self, order: &str) -> &str {
        match order {
            "name" => "Name",
            "url" => "Url",
            "homepage" => "Homepage",
            "favicon" => "Favicon",
            "tags" => "Tags",
            "country" => "Country",
            "state" => "Subcountry",
            "language" => "Language",
            "votes" => "Votes",
            "negativevotes" => "NegativeVotes",
            "codec" => "Codec",
            "bitrate" => "Bitrate",
            "lastcheckok" => "LastCheckOK",
            "lastchecktime" => "LastCheckTime",
            "clicktimestamp" => "ClickTimestamp",
            "clickcount" => "clickcount",
            "clicktrend" => "ClickTrend",
            _ => "Name",
        }
    }

    pub fn get_stations_by_column(&self, column_name: &str, search: String, exact: bool, order: &str, reverse: bool, hidebroken: bool, offset: u32, limit: u32) -> Vec<Station> {
        let order = self.filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
        let query: String = if exact {
            format!("SELECT {columns} from Station WHERE {column_name}=? {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = Connection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        }else{
            format!("SELECT {columns} from Station WHERE {column_name} LIKE CONCAT('%',?,'%') {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = Connection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        };
        let results = self.pool.prep_exec(query, (search,));
        self.get_stations(results)
    }

    pub fn get_stations_by_column_multiple(&self, column_name: &str, search: String, exact: bool, order: &str, reverse: bool, hidebroken: bool, offset: u32, limit: u32) -> Vec<Station> {
        let order = self.filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
        let query: String = if exact {
            format!(r"SELECT {columns} from Station WHERE ({column_name}=?
             OR {column_name} LIKE CONCAT('%,',?,',%')
             OR {column_name} LIKE CONCAT(?,',%')
             OR {column_name} LIKE CONCAT('%,',?))
             {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = Connection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        }else{
            format!("SELECT {columns} from Station WHERE {column_name} LIKE CONCAT('%',?,'%') {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = Connection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        };
        let results = if exact { self.pool.prep_exec(query, (&search,&search,&search,&search,)) } else { self.pool.prep_exec(query, (search,)) };
        self.get_stations(results)
    }

    pub fn get_stations_by_id(&self, id: i32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT {columns} from Station WHERE StationID='{id}' ORDER BY Name", columns = Connection::COLUMNS, id = id);
        self.get_stations_query(query)
    }

    pub fn get_stations_topvote(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT {columns} from Station ORDER BY Votes DESC LIMIT {limit}", columns = Connection::COLUMNS, limit = limit);
        self.get_stations_query(query)
    }

    pub fn get_stations_topclick(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT {columns} from Station ORDER BY clickcount DESC LIMIT {limit}", columns = Connection::COLUMNS, limit = limit);
        self.get_stations_query(query)
    }

    pub fn get_stations_lastclick(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT {columns} from Station ORDER BY ClickTimestamp DESC LIMIT {limit}", columns = Connection::COLUMNS, limit = limit);
        self.get_stations_query(query)
    }

    pub fn get_stations_lastchange(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT {columns} from Station ORDER BY Creation DESC LIMIT {limit}", columns = Connection::COLUMNS, limit = limit);
        self.get_stations_query(query)
    }

    pub fn get_changes(&self, _uuid: Option<String>, seconds: u32) -> Vec<StationHistory> {
        let query : String;
        let seconds_str: String = if seconds > 0 { format!(" AND TIME_TO_SEC(TIMEDIFF(Now(),Creation))<{}",seconds) } else { "".to_string() };
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from StationHistory WHERE 1=1 {seconds} ORDER BY Creation DESC", seconds = seconds_str);
        self.get_stations_history(query)
    }

    fn get_stations_query(&self, query: String) -> Vec<Station> {
        let results = self.pool.prep_exec(query, ());
        self.get_stations(results)
    }

    fn get_stations(&self, results: self::mysql::Result<QueryResult<'static>>) -> Vec<Station> {
        let mut stations: Vec<Station> = vec![];
        for result in results {
            for row_ in result {
                let mut row = row_.unwrap();
                let s = Station {
                    id:              row.take("StationID").unwrap(),
                    changeuuid:      row.take("ChangeUuid").unwrap_or("".to_string()),
                    stationuuid:     row.take("StationUuid").unwrap_or("".to_string()),
                    name:            row.take("Name").unwrap_or("".to_string()),
                    url:             row.take("Url").unwrap_or("".to_string()),
                    homepage:        row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    favicon:         row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    tags:            row.take_opt("Tags").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    country:         row.take_opt("Country").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    state:           row.take_opt("Subcountry").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    language:        row.take_opt("Language").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    votes:           row.take_opt("Votes").unwrap_or(Ok(0)).unwrap_or(0),
                    negativevotes:   row.take_opt("NegativeVotes").unwrap_or(Ok(0)).unwrap_or(0),
                    lastchangetime:  row.take_opt("CreationFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    ip:              row.take_opt("Ip").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    codec:           row.take_opt("Codec").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    bitrate:         row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
                    hls:             row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0),
                    lastcheckok:     row.take_opt("LastCheckOK").unwrap_or(Ok(0)).unwrap_or(0),
                    lastchecktime:   row.take_opt("LastCheckTimeFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    lastcheckoktime: row.take_opt("LastCheckOkTimeFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    clicktimestamp:  row.take_opt("ClickTimestampFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    clickcount:      row.take_opt("clickcount").unwrap_or(Ok(0)).unwrap_or(0),
                    clicktrend:      row.take_opt("ClickTrend").unwrap_or(Ok(0)).unwrap_or(0)
                };
                stations.push(s);
            }
        }

        stations
    }

    fn get_stations_history(&self, query: String) -> Vec<StationHistory> {
        let mut changes: Vec<StationHistory> = vec![];
        let results = self.pool.prep_exec(query, ());
        for result in results {
            for row_ in result {
                let mut row = row_.unwrap();
                let s = StationHistory {
                    id:              row.take("StationID").unwrap(),
                    changeuuid:      row.take("ChangeUuid").unwrap_or("".to_string()),
                    stationuuid:     row.take("StationUuid").unwrap_or("".to_string()),
                    name:            row.take("Name").unwrap_or("".to_string()),
                    url:             row.take("Url").unwrap_or("".to_string()),
                    homepage:        row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    favicon:         row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    tags:            row.take_opt("Tags").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    country:         row.take_opt("Country").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    state:           row.take_opt("Subcountry").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    language:        row.take_opt("Language").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    votes:           row.take_opt("Votes").unwrap_or(Ok(0)).unwrap_or(0),
                    negativevotes:   row.take_opt("NegativeVotes").unwrap_or(Ok(0)).unwrap_or(0),
                    lastchangetime:  row.take_opt("Creation").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    ip:              row.take_opt("Ip").unwrap_or(Ok("".to_string())).unwrap_or("".to_string())
                };
                changes.push(s);
            }
        }

        changes
    }

    pub fn get_1_n(&self, column: &str, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<Result1n>{
        let query : String;
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
        let result = match search{
            Some(value) => {
                query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM Station WHERE {column} LIKE CONCAT('%',?,'%') AND {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
                self.pool.prep_exec(query, (value,))
            },
            None => {
                query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM Station WHERE {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
                self.pool.prep_exec(query, ())
            }
        };

        let stations: Vec<Result1n> =
        result.map(|result| {
            result.map(|x| x.unwrap()).map(|row| {
                let (name, value, stationcount) = mysql::from_row(row);
                Result1n {
                    name: name,
                    value: value,
                    stationcount: stationcount,
                }
            }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
        }).unwrap(); // Unwrap `Vec<Payment>`
        stations
    }

    pub fn get_states(&self, country: Option<String>, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<State>{
        let mut params: Vec<Value> = Vec::with_capacity(1);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
        let country_string = match country {
            Some(c) => {
                params.push(c.into());
                format!(" AND Country=?")
            },
            None => "".to_string()
        };
        let search_string = match search {
            Some(c) => {
                params.push((format!("%{}%",c)).into());
                format!(" AND Subcountry LIKE ?")
            },
            None => "".to_string()
        };
        
        let mut my_stmt = self.pool.prepare(format!(r"SELECT Subcountry AS value,Subcountry,Country,COUNT(*) AS stationcount FROM Station WHERE Subcountry <> '' {country} {search} {hidebroken} GROUP BY Subcountry, Country ORDER BY {order} {reverse}",hidebroken = hidebroken_string, order = order, country = country_string, reverse = reverse_string, search = search_string)).unwrap();
        let my_results = my_stmt.execute(params);
        let mut states: Vec<State> = vec![];

        for my_result in my_results {
            for my_row in my_result {
                let mut row_unwrapped = my_row.unwrap();
                states.push(State{
                    name: row_unwrapped.take(0).unwrap_or("".into()),
                    value: row_unwrapped.take(1).unwrap_or("".into()),
                    country: row_unwrapped.take(2).unwrap_or("".into()),
                    stationcount: row_unwrapped.take(3).unwrap_or(0)
                });
            }
        };
        states
    }

    pub fn init_tables(&self) {
        let result = self.pool.prep_exec(
            r#"CREATE OR REPLACE TABLE TagCache(TagName VARCHAR(100) COLLATE utf8_bin NOT NULL,
            Primary Key (TagName),
            StationCount INT DEFAULT 0,
            StationCountWorking INT DEFAULT 0) CHARSET=utf8 COLLATE=utf8_bin"#,
            ());
        match result {
            Ok(_) => {},
            Err(err) => {println!("{}",err);}
        }
    }

    pub fn get_extra(&self, table_name: &str, column_name: &str, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<ExtraInfo>{
        let mut params: Vec<Value> = Vec::with_capacity(1);
        let mut items = vec![];
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
        let search_string = match search {
            Some(c) => {
                params.push((format!("%{}%",c)).into());
                format!(" AND {} LIKE ?", column_name)
            },
            None => "".to_string()
        };
        let mut stmt = self.pool.prepare(format!("SELECT {column_name} AS value, {column_name}, StationCount as stationcount, StationCountWorking FROM {table_name} WHERE {column_name} <> '' {search} {hidebroken} ORDER BY {order} {reverse}",search = search_string, order = order, reverse = reverse_string, hidebroken = hidebroken_string, table_name = table_name, column_name = column_name)).unwrap();
        let my_results = stmt.execute(params);
        for my_result in my_results {
            for my_row in my_result {
                let mut row_unwrapped = my_row.unwrap();
                items.push(ExtraInfo{
                    name: row_unwrapped.take(0).unwrap_or("".into()),
                    value: row_unwrapped.take(1).unwrap_or("".into()),
                    stationcount: row_unwrapped.take(2).unwrap_or(0),
                    stationcountworking: row_unwrapped.take(3).unwrap_or(0)
                });
            }
        }
        items
    }
}
pub enum DBError{
    ConnectionError (String),
    EncodeError (String),
}

impl std::fmt::Display for DBError{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        match *self{
            DBError::ConnectionError(ref v) => write!(f, "{}", v),
            DBError::EncodeError(ref v) => write!(f, "{}", v),
        }
    }
}

fn get_cached_items(pool: &mysql::Pool, table_name: &str, column_name: &str) -> HashMap<String,u32>{
    let mut items = HashMap::new();
    let mut my_stmt = pool.prepare(format!("SELECT {column_name},StationCount FROM {table_name}", table_name = table_name, column_name = column_name)).unwrap();
    let my_results = my_stmt.execute(());

    for my_result in my_results {
        for my_row in my_result {
            let mut row_unwrapped = my_row.unwrap();
            let key : String = row_unwrapped.take(0).unwrap_or("".into());
            let value : u32 = row_unwrapped.take(1).unwrap_or(0);
            let lower = key.to_lowercase();
            items.insert(lower,value);
        }
    };
    items
}

fn get_stations_multi_items(pool: &mysql::Pool, column_name: &str) -> HashMap<String,u32>{
    let mut items = HashMap::new();
    let mut my_stmt = pool.prepare(format!("SELECT {column_name} FROM Station",column_name = column_name)).unwrap();
    let my_results = my_stmt.execute(());

    for my_result in my_results {
        for my_row in my_result {
            let mut row_unwrapped = my_row.unwrap();
            let tags_str : String = row_unwrapped.take(0).unwrap_or("".into());
            let tags_arr = tags_str.split(',');
            for single_tag in tags_arr {
                let single_tag_trimmed = single_tag.trim().to_lowercase();
                if single_tag_trimmed != "" {
                    let counter = items.entry(single_tag_trimmed).or_insert(0);
                    *counter += 1;
                }
            }
        }
    };
    items
}

fn update_cache_item(pool: &mysql::Pool, tag: &String, count: u32, table_name: &str, column_name: &str){
    let mut my_stmt = pool.prepare(format!(r"UPDATE {table_name} SET StationCount=? WHERE {column_name}=?",table_name = table_name, column_name = column_name)).unwrap();
    let params = (count,tag);
    let result = my_stmt.execute(params);
    match result {
        Ok(_) => {},
        Err(err) => {println!("{}",err);}
    }
}

fn insert_to_cache(pool: &mysql::Pool, tags: HashMap<&String, u32>, table_name: &str, column_name: &str){
    let query = format!("INSERT INTO {table_name}({column_name},StationCount) VALUES(?,?)", table_name = table_name, column_name = column_name);
    let mut my_stmt = pool.prepare(query.trim_matches(',')).unwrap();
    for item in tags.iter() {
        let result = my_stmt.execute((item.0,item.1));
        match result {
            Ok(_) => {},
            Err(err) => {println!("{}",err);}
        }
    }
}

fn remove_from_cache(pool: &mysql::Pool, tags: Vec<&String>, table_name: &str, column_name: &str){
    let mut query = format!("DELETE FROM {table_name} WHERE {column_name}=''", table_name = table_name, column_name = column_name);
    for _ in 0..tags.len() {
        query.push_str(" OR ");
        query.push_str(column_name);
        query.push_str("=?");
    }
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(tags);
    match result {
        Ok(_) => {},
        Err(err) => {println!("{}",err);}
    }
}

pub fn refresh_cache_items(pool: &mysql::Pool, cache_table_name: &str, cache_column_name: &str, station_column_name: &str){
    let items_cached = get_cached_items(pool, cache_table_name, cache_column_name);
    let items_current = get_stations_multi_items(pool, station_column_name);
    let mut changed = 0;

    let mut to_delete = vec![];
    for item_cached in items_cached.keys() {
        if ! items_current.contains_key(item_cached){
            to_delete.push(item_cached);
        }
    }
    remove_from_cache(pool, to_delete, cache_table_name, cache_column_name);

    let mut to_insert: HashMap<&String,u32> = HashMap::new();
    for item_current in items_current.keys() {
        if ! items_cached.contains_key(item_current){
            //self.insert_tag(tag_current, *tags_current.get(tag_current).unwrap_or(&0));
            to_insert.insert(item_current, *items_current.get(item_current).unwrap_or(&0));
        } else {
            let value_new = *items_current.get(item_current).unwrap_or(&0);
            let value_old = *items_cached.get(item_current).unwrap_or(&0);
            if value_old != value_new {
                update_cache_item(pool, item_current, value_new, cache_table_name, cache_column_name);
                changed = changed + 1;
            }
        }
    }
    insert_to_cache(pool, to_insert, cache_table_name, cache_column_name);
    println!("{}: {} -> {}, Changed: {}", station_column_name, items_cached.len(), items_current.len(), changed);
    //let to_add = tags_stations.difference(&tags_cached);
    /*for item_to_add in to_add {
        self.insert_tag(item_to_add);
    }*/
    /*let x = to_add.collect::<Vec<&String>>();
    self.insert_tags(x);
    let to_delete = tags_cached.difference(&tags_stations);
    for item_to_delete in to_delete {
        self.remove_tag(item_to_delete);
    }*/
}

fn start_refresh_worker(connection_string: String, update_caches_interval: u64){
    thread::spawn(move || {
        loop{
            let pool = mysql::Pool::new(&connection_string);
            match pool {
                Ok(p) => {
                    //println!("REFRESH START");
                    refresh_cache_items(&p, "TagCache", "TagName", "Tags");
                    refresh_cache_items(&p, "LanguageCache", "LanguageName", "Language");
                    //println!("REFRESH END");
                },
                Err(e) => println!("{}",e)
            }
            
            thread::sleep(::std::time::Duration::new(update_caches_interval,0));
        }
    });
}

pub fn new(connection_string: &String, update_caches_interval: u64) -> Result<Connection, DBError> {
    let connection_string2 = connection_string.clone();
    println!("Connection string: {}", connection_string);
    
    let pool = mysql::Pool::new(connection_string);
    match pool {
        Ok(p) => {
            let c = Connection{pool: p};
            c.init_tables();

            if update_caches_interval > 0 {
                start_refresh_worker(connection_string2, update_caches_interval);
            }

            Ok(c)
            },
        Err(e) => Err(DBError::ConnectionError(e.to_string()))
    }
}