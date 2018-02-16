extern crate mysql;
extern crate xml_writer;
extern crate chrono;

use std::collections::HashMap;
use db::mysql::Value;
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
    lastchangetime: chrono::NaiveDateTime,
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
pub struct Tag {
    name: String,
    value: String,
    stationcount: u32,
    stationcountworking: u32
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
    Ok(String::from_utf8(xml.into_inner()).unwrap())
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
    Ok(String::from_utf8(xml.into_inner()).unwrap())
}

pub fn serialize_tag_list(entries: Vec<Tag>) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    for entry in entries{
        xml.begin_elem("tag")?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("value", &entry.value)?;
            let count_str = format!("{}", entry.stationcount);
            xml.attr_esc("stationcount", &count_str)?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap())
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
            xml.attr_esc("codec", "MP3")?;
            xml.attr_esc("bitrate", "0")?;
            xml.attr_esc("hls", "0")?;
            xml.attr_esc("lastcheckok", "1")?;
            xml.attr_esc("lastchecktime", "2018-02-16 02:20:30")?;
            xml.attr_esc("lastcheckoktime", "2018-02-16 02:20:30")?;
            xml.attr_esc("clicktimestamp", "2018-02-16 02:20:30")?;
            xml.attr_esc("clickcount", "0")?;
            xml.attr_esc("clicktrend", "0")?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap())
}


impl Connection {
    pub fn get_stations_by_all(&self) -> Vec<Station> {
        let query : String;
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from Station ORDER BY Name");
        self.get_stations(query)
    }

    pub fn get_stations_by_name(&self, search: String) -> Vec<Station> {
        let query : String;
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from Station WHERE Name LIKE '%{search}%' ORDER BY Name", search = search);
        self.get_stations(query)
    }

    pub fn get_stations_by_id(&self, id: i32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from Station WHERE StationID='{id}' ORDER BY Name", id = id);
        self.get_stations(query)
    }

    pub fn get_stations_topvote(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from Station ORDER BY Votes DESC LIMIT {limit}", limit = limit);
        self.get_stations(query)
    }

    pub fn get_stations_topclick(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from Station ORDER BY clickcount DESC LIMIT {limit}", limit = limit);
        self.get_stations(query)
    }

    pub fn get_stations_lastclick(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from Station ORDER BY ClickTimestamp DESC LIMIT {limit}", limit = limit);
        self.get_stations(query)
    }

    pub fn get_stations_lastchange(&self, limit: u32) -> Vec<Station> {
        let query : String;
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from Station ORDER BY Creation DESC LIMIT {limit}", limit = limit);
        self.get_stations(query)
    }

    pub fn get_changes(&self, uuid: Option<String>, seconds: u32) -> Vec<Station> {
        let query : String;
        let seconds_str: String = if seconds > 0 { format!(" AND TIME_TO_SEC(TIMEDIFF(Now(),Creation))<{}",seconds) } else { "".to_string() };
        query = format!("SELECT StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,Tags,Country,Subcountry,Language,Votes,NegativeVotes,Creation,Ip from StationHistory WHERE 1=1 {seconds} ORDER BY Creation DESC", seconds = seconds_str);
        self.get_stations(query)
    }

    fn get_stations(&self, query: String) -> Vec<Station> {
        let mut stations: Vec<Station> = vec![];
        let results = self.pool.prep_exec(query, ());
        for result in results {
            for row_ in result {
                let mut row = row_.unwrap();
                let s = Station {
                    id: row.take("StationID").unwrap(),
                    changeuuid: row.take("ChangeUuid").unwrap_or("".to_string()),
                    stationuuid: row.take("StationUuid").unwrap_or("".to_string()),
                    name: row.take("Name").unwrap_or("".to_string()),
                    url: row.take("Url").unwrap_or("".to_string()),
                    homepage: row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    favicon: row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    tags: row.take_opt("Tags").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    country: row.take_opt("Country").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    state: row.take_opt("Subcountry").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    language: row.take_opt("Language").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    votes: row.take("Votes").unwrap_or(0),
                    negativevotes: row.take("NegativeVotes").unwrap_or(0),
                    lastchangetime: row.take("Creation").unwrap(),
                    ip: row.take_opt("Ip").unwrap_or(Ok("".to_string())).unwrap_or("".to_string())
                };
                stations.push(s);
            }
        }

        stations
    }

    pub fn get_1_n(&self, column: &str, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<Result1n>{
        let query : String;
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
        match search{
            Some(value) => {
                query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM Station WHERE {column} LIKE '%{search}%' AND {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, search = value, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
            },
            None => {
                query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM Station WHERE {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
            }
        }

        let stations: Vec<Result1n> =
        self.pool.prep_exec(query, ())
        .map(|result| {
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
            "CREATE OR REPLACE TABLE TagCache(TagName VARCHAR(100) COLLATE utf8_bin NOT NULL,
            Primary Key (TagName),
            StationCount INT DEFAULT 0,
            StationCountWorking INT DEFAULT 0) CHARSET=utf8 COLLATE=utf8_bin",
            ());
        match result {
            Ok(_) => {},
            Err(err) => {println!("{}",err);}
        }
    }

    pub fn get_tags(&self, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<Tag>{
        let mut params: Vec<Value> = Vec::with_capacity(1);
        let mut tags = vec![];
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
        let search_string = match search {
            Some(c) => {
                params.push((format!("%{}%",c)).into());
                format!(" AND TagName LIKE ?")
            },
            None => "".to_string()
        };
        let mut stmt = self.pool.prepare(format!("SELECT TagName AS value, TagName, StationCount as stationcount, StationCountWorking FROM TagCache WHERE TagName <> '' {search} {hidebroken} ORDER BY {order} {reverse}",search = search_string, order = order, reverse = reverse_string, hidebroken = hidebroken_string)).unwrap();
        let my_results = stmt.execute(params);
        for my_result in my_results {
            for my_row in my_result {
                let mut row_unwrapped = my_row.unwrap();
                tags.push(Tag{
                    name: row_unwrapped.take(0).unwrap_or("".into()),
                    value: row_unwrapped.take(1).unwrap_or("".into()),
                    stationcount: row_unwrapped.take(2).unwrap_or(0),
                    stationcountworking: row_unwrapped.take(3).unwrap_or(0)
                });
            }
        }
        tags
    }
}
pub enum DBError{
    ConnectionError (String)
}

impl std::fmt::Display for DBError{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        match *self{
            DBError::ConnectionError(ref v) => write!(f, "{}", v)
        }
    }
}

fn get_cached_tags(pool: &mysql::Pool) -> HashMap<String,u32>{
    let mut tags = HashMap::new();
    let mut my_stmt = pool.prepare("SELECT TagName,StationCount FROM TagCache").unwrap();
    let my_results = my_stmt.execute(());

    for my_result in my_results {
        for my_row in my_result {
            let mut row_unwrapped = my_row.unwrap();
            let key : String = row_unwrapped.take(0).unwrap_or("".into());
            let value : u32 = row_unwrapped.take(1).unwrap_or(0);
            let lower = key.to_lowercase();
            tags.insert(lower,value);
        }
    };
    tags
}

fn get_stations_tags(pool: &mysql::Pool) -> HashMap<String,u32>{
    let mut tags = HashMap::new();
    let mut my_stmt = pool.prepare("SELECT Tags FROM Station").unwrap();
    let my_results = my_stmt.execute(());

    for my_result in my_results {
        for my_row in my_result {
            let mut row_unwrapped = my_row.unwrap();
            let tags_str : String = row_unwrapped.take(0).unwrap_or("".into());
            let tags_arr = tags_str.split(',');
            for single_tag in tags_arr {
                let single_tag_trimmed = single_tag.trim().to_lowercase();
                if single_tag_trimmed != "" {
                    let counter = tags.entry(single_tag_trimmed).or_insert(0);
                    *counter += 1;
                }
            }
        }
    };
    tags
}

fn update_tag(pool: &mysql::Pool, tag: &String, count: u32){
    let mut my_stmt = pool.prepare(r"UPDATE TagCache SET StationCount=? WHERE TagName=?").unwrap();
    let params = (count,tag);
    let result = my_stmt.execute(params);
    match result {
        Ok(_) => {},
        Err(err) => {println!("{}",err);}
    }
}

fn insert_tags(pool: &mysql::Pool, tags: HashMap<&String, u32>){
    let query = String::from("INSERT INTO TagCache(TagName,StationCount) VALUES(?,?)");
    let mut my_stmt = pool.prepare(query.trim_matches(',')).unwrap();
    for item in tags.iter() {
        let result = my_stmt.execute((item.0,item.1));
        match result {
            Ok(_) => {},
            Err(err) => {println!("{}",err);}
        }
    }
}

fn remove_tags(pool: &mysql::Pool, tags: Vec<&String>){
    let mut query = String::from("DELETE FROM TagCache WHERE TagName=''");
    for _ in 0..tags.len() {
        query.push_str(" OR TagName=?");
    }
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(tags);
    match result {
        Ok(_) => {},
        Err(err) => {println!("{}",err);}
    }
}

pub fn refresh_cache_tags(pool: &mysql::Pool){
    let tags_cached = get_cached_tags(pool);
    let tags_current = get_stations_tags(pool);
    let mut changed = 0;

    let mut to_delete = vec![];
    for tag_cached in tags_cached.keys() {
        if ! tags_current.contains_key(tag_cached){
            to_delete.push(tag_cached);
        }
    }
    remove_tags(pool, to_delete);

    let mut to_insert: HashMap<&String,u32> = HashMap::new();
    for tag_current in tags_current.keys() {
        if ! tags_cached.contains_key(tag_current){
            //self.insert_tag(tag_current, *tags_current.get(tag_current).unwrap_or(&0));
            to_insert.insert(tag_current, *tags_current.get(tag_current).unwrap_or(&0));
        } else {
            let value_new = *tags_current.get(tag_current).unwrap_or(&0);
            let value_old = *tags_cached.get(tag_current).unwrap_or(&0);
            if value_old != value_new {
                update_tag(pool, tag_current, value_new);
                changed = changed + 1;
            }
        }
    }
    insert_tags(pool, to_insert);
    println!("Tags: {} -> {}, Changed: {}", tags_cached.len(), tags_current.len(), changed);
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

fn start_refresh_worker(connection_string: String){
    thread::spawn(move || {
        loop{
            let pool = mysql::Pool::new(&connection_string);
            match pool {
                Ok(p) => {
                    println!("REFRESH START");
                    refresh_cache_tags(&p);
                    println!("REFRESH END");
                },
                Err(e) => println!("{}",e)
            }
            
            thread::sleep(::std::time::Duration::new(10,0));
        }
    });
}

pub fn new(host: &String,port : i32, name: &String, user: &String, password: &String) -> Result<Connection, DBError> {
    let connection_string = format!("mysql://{}:{}@{}:{}/{}",user,password,host,port,name);
    let connection_string2 = connection_string.clone();
    println!("Connection string: {}", connection_string);
    
    let pool = mysql::Pool::new(connection_string);
    match pool {
        Ok(p) => {
            let c = Connection{pool: p};
            c.init_tables();

            start_refresh_worker(connection_string2);

            Ok(c)
            },
        Err(e) => Err(DBError::ConnectionError(e.to_string()))
    }
}