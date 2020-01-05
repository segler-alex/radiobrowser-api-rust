extern crate chrono;
extern crate xml_writer;

use crate::api::data::StationAddResult;
use mysql::Value;
use std;
extern crate uuid;
use self::uuid::Uuid;

#[derive(Clone)]
pub struct Connection {
    pool: mysql::Pool,
}

impl Connection {
    fn fix_multi_field(value: &str) -> String {
        let values: Vec<String> = value.split(",").map(|v| v.trim().to_lowercase().to_string()).collect();
        values.join(",")
    }

    pub fn add_station_opt(&self, name: Option<String>, url: Option<String>, homepage: Option<String>, favicon: Option<String>,
                        country: Option<String>, countrycode: Option<String>, state: Option<String>, language: Option<String>, tags: Option<String>) -> StationAddResult{
        let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,CountryCode,Subcountry,Language,Tags,ChangeUuid,StationUuid, UrlCache) 
                                VALUES(:name, :url, :homepage, :favicon, :country, :countrycode, :state, :language, :tags, :changeuuid, :stationuuid, '')");
        
        if name.is_none(){
            return StationAddResult::new_err("name is empty");
        }
        if url.is_none(){
            return StationAddResult::new_err("url is empty");
        }
        let name = name.unwrap();
        if name.len() > 400{
            return StationAddResult::new_err("name is longer than 400 chars");
        }

        let stationuuid = Uuid::new_v4().to_hyphenated().to_string();
        let changeuuid = Uuid::new_v4().to_hyphenated().to_string();
        let params = params!{
            "name" => name,
            "url" => url.unwrap(),
            "homepage" => homepage.unwrap_or_default(),
            "favicon" => favicon.unwrap_or_default(),
            "country" => country.unwrap_or_default(),
            "countrycode" => countrycode.unwrap_or_default(),
            "state" => state.unwrap_or_default(),
            "language" => Connection::fix_multi_field(&language.unwrap_or_default()),
            "tags" => Connection::fix_multi_field(&tags.unwrap_or_default()),
            "changeuuid" => changeuuid,
            "stationuuid" => stationuuid.clone(),
        };

        let results = self.pool.prep_exec(query, params);
        match results {
            Ok(_) => {
                let backup_result = self.backup_stations_by_uuid(&(vec![stationuuid.clone()]));
                match backup_result {
                    Ok(_) => StationAddResult::new_ok(stationuuid),
                    Err(err) => StationAddResult::new_err(&err.to_string())
                }
            },
            Err(err)=>StationAddResult::new_err(&err.to_string())
        }
    }

    fn backup_stations_by_uuid(&self, stationuuids: &Vec<String>) -> Result<(),Box<dyn std::error::Error>>{
        let mut insert_params: Vec<Value> = vec![];
        let mut insert_query = vec![];
        for stationuuid in stationuuids {
            insert_params.push(stationuuid.into());
            insert_query.push("?");
        }

        let query = format!("INSERT INTO StationHistory(StationID,Name,Url,Homepage,Favicon,Country,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid)
                                                 SELECT StationID,Name,Url,Homepage,Favicon,Country,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid FROM Station WHERE StationUuid IN ({})", insert_query.join(","));
        let mut stmt = self.pool.prepare(query)?;
        stmt.execute(insert_params)?;
        Ok(())
    }
}

pub fn new(connection_string: &String) -> Result<Connection, Box<dyn std::error::Error>> {
    let pool = mysql::Pool::new(connection_string)?;
    Ok(Connection { pool })
}
