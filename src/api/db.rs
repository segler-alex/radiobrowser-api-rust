extern crate chrono;
extern crate xml_writer;

use api::data::StationHistoryCurrent;
use api::data::StationAddResult;
use api::data::Station;
use api::data::Result1n;
use api::data::ExtraInfo;
use api::data::State;
use api::data::StationCheck;
use mysql::QueryResult;
use mysql::Value;
use std;
use std::collections::HashMap;
use thread;
extern crate uuid;
use self::uuid::Uuid;
use api::simple_migrate::Migrations;
use api::api_error;

#[derive(Clone)]
pub struct Connection {
    pool: mysql::Pool,
}

impl Connection {
    const COLUMNS: &'static str =
        "StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,UrlCache,
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

    const COLUMNS_CHECK: &'static str =
        "CheckID, StationUuid, CheckUuid, Source, Codec, Bitrate, Hls, CheckOK,
    CheckTime,
    Date_Format(CheckTime,'%Y-%m-%d %H:%i:%s') AS CheckTimeFormated,
    UrlCache";

    pub fn get_single_column_number(&self, query: &str) -> Result<u64,Box<std::error::Error>> {
        let results = self.pool.prep_exec(query, ())?;
        self.get_single_column_number_intern(results)
    }

    pub fn get_single_column_number_intern(&self, mut results: QueryResult<'static>) -> Result<u64,Box<std::error::Error>> {
        let mut result_row = results.next().unwrap()?;
        let count: u64 = result_row.take(0).unwrap();
        Ok(count)
    }

    pub fn get_station_count(&self) -> u64 {
        self.get_single_column_number(r#"SELECT COUNT(*) AS StationCount FROM Station WHERE LastCheckOK=True"#).unwrap_or(0)
    }

    pub fn get_broken_station_count(&self) -> u64 {
        self.get_single_column_number(r#"SELECT COUNT(*) AS StationCount FROM Station WHERE LastCheckOK=False"#).unwrap_or(0)
    }

    pub fn get_tag_count(&self) -> u64 {
        self.get_single_column_number(r#"SELECT COUNT(*) AS StationCount FROM TagCache"#).unwrap_or(0)
    }

    pub fn get_country_count(&self) -> u64 {
        self.get_single_column_number(r#"SELECT COUNT(DISTINCT(Country)) AS StationCount FROM Station"#).unwrap_or(0)
    }

    pub fn get_language_count(&self) -> u64 {
        self.get_single_column_number(r#"SELECT COUNT(*) AS StationCount FROM LanguageCache"#).unwrap_or(0)
    }

    pub fn get_click_count_last_hour(&self) -> u64 {
        self.get_single_column_number(r#"SELECT COUNT(*) FROM StationClick WHERE TIMESTAMPDIFF(MINUTE,ClickTimestamp,now())<=60;"#).unwrap_or(0)
    }

    pub fn get_click_count_last_day(&self) -> u64 {
        self.get_single_column_number(r#"SELECT COUNT(*) FROM StationClick WHERE TIMESTAMPDIFF(HOUR,ClickTimestamp,now())<=24;"#).unwrap_or(0)
    }

    /*pub fn is_empty(&self) -> Result<bool, Box<std::error::Error>> {
        let count = self.get_single_column_number(r#"SELECT COUNT(*) FROM StationHistory"#)?;
        Ok(count == 0)
    }*/

    pub fn update_station(&self, station: Station) -> Result<(),Box<std::error::Error>> {
        let query = format!("UPDATE Station SET Name=:name,Url=:url,Homepage=:homepage,
            Favicon=:favicon,Country=:country,Subcountry=:state,Language=:language,
            Tags=:tags,ChangeUuid=:changeuuid,UrlCache=:urlcache
            WHERE StationUuid=:stationuuid");
        let params = params!{
            "name" => station.name,
            "url" => station.url,
            "homepage" => station.homepage,
            "favicon" => station.favicon,
            "country" => station.country,
            "state" => station.state,
            "language" => station.language,
            "tags" => station.tags,
            "changeuuid" => station.changeuuid,
            "stationuuid" => &station.stationuuid,
            "urlcache" => "",
        };

        self.pool.prep_exec(query, params)?;
        self.backup_station_by_uuid(&station.stationuuid)?;

        Ok(())
    }

    pub fn add_station(&self, station: Station) -> Result<u64,Box<std::error::Error>> {
        let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,Subcountry,Language,Tags,ChangeUuid,StationUuid, UrlCache) 
                                VALUES(:name, :url, :homepage, :favicon, :country, :state, :language, :tags, :changeuuid, :stationuuid, '')");
        let params = params!{
            "name" => station.name,
            "url" => station.url,
            "homepage" => station.homepage,
            "favicon" => station.favicon,
            "country" => station.country,
            "state" => station.state,
            "language" => station.language,
            "tags" => station.tags,
            "changeuuid" => station.changeuuid,
            "stationuuid" => station.stationuuid,
        };

        let results = self.pool.prep_exec(query, params)?;
        let id = results.last_insert_id();
        self.backup_station_by_id(id)?;
        Ok(id)
    }

    pub fn add_station_opt(&self, name: Option<String>, url: Option<String>, homepage: Option<String>, favicon: Option<String>,
                        country: Option<String>, state: Option<String>, language: Option<String>, tags: Option<String>) -> StationAddResult{
        let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,Subcountry,Language,Tags,ChangeUuid,StationUuid, UrlCache) 
                                VALUES(:name, :url, :homepage, :favicon, :country, :state, :language, :tags, :changeuuid, :stationuuid, '')");
        
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
            "state" => state.unwrap_or_default(),
            "language" => language.unwrap_or_default(),
            "tags" => tags.unwrap_or_default(),
            "changeuuid" => changeuuid,
            "stationuuid" => stationuuid.clone(),
        };

        let results = self.pool.prep_exec(query, params);
        match results {
            Ok(results) => {
                let id = results.last_insert_id();
                let backup_result = self.backup_station_by_id(id);
                match backup_result {
                    Ok(_) => StationAddResult::new_ok(id, stationuuid),
                    Err(err) => StationAddResult::new_err(&err.to_string())
                }
            },
            Err(err)=>StationAddResult::new_err(&err.to_string())
        }
    }

    fn backup_station_by_id(&self, stationid: u64) -> Result<(),Box<std::error::Error>>{
        let query = format!("INSERT INTO StationHistory(StationID,Name,Url,Homepage,Favicon,Country,SubCountry,Language,Tags,Votes,NegativeVotes,Creation,IP,StationUuid,ChangeUuid)
                                SELECT StationID,Name,Url,Homepage,Favicon,Country,SubCountry,Language,Tags,Votes,NegativeVotes,Creation,IP,StationUuid,ChangeUuid FROM Station WHERE StationID=:id");
        let params = params!{
            "id" => stationid,
        };

        self.pool.prep_exec(query, params)?;
        
        Ok(())
    }

    fn backup_station_by_uuid(&self, stationuuid: &str) -> Result<(),Box<std::error::Error>>{
        let query = format!("INSERT INTO StationHistory(StationID,Name,Url,Homepage,Favicon,Country,SubCountry,Language,Tags,Votes,NegativeVotes,Creation,IP,StationUuid,ChangeUuid)
                                SELECT StationID,Name,Url,Homepage,Favicon,Country,SubCountry,Language,Tags,Votes,NegativeVotes,Creation,IP,StationUuid,ChangeUuid FROM Station WHERE StationUuid=:stationuuid");
        let params = params!{
            "stationuuid" => stationuuid,
        };

        self.pool.prep_exec(query, params)?;
        
        Ok(())
    }

    /*pub fn insert_station_change(&self, stationchange: &StationHistoryCurrent) -> Result<(),Box<std::error::Error>> {
        let params = params!{
            "stationid" => stationchange.id,
            "name" => stationchange.name.clone(),
            "url" => stationchange.url.clone(),
            "homepage" => stationchange.homepage.clone(),
            "favicon" => stationchange.favicon.clone(),
            "country" => stationchange.country.clone(),
            "state" => stationchange.state.clone(),
            "language" => stationchange.language.clone(),
            "tags" => stationchange.tags.clone(),
            "ip" => stationchange.ip.clone(),
            "stationuuid" => stationchange.stationuuid.clone(),
            "changeuuid" => stationchange.changeuuid.clone(),
        };
        let query = format!("INSERT INTO
            StationHistory(StationID,Name,Url,Homepage,Favicon,Country,SubCountry,Language,Tags,IP,StationUuid,ChangeUuid)
            VALUES(:stationid,:name,:url,:homepage,:favicon,:country,:state,:language,:tags,:ip,:stationuuid,:changeuuid)");
        self.pool.prep_exec(query, params)?;
        Ok(())
    }*/

    /*pub fn insert_station_changes(&self, stations: &[StationHistoryCurrent]) -> Result<(),Box<std::error::Error>> {
        let mut params = params!{
            "x" => "x"
        };
        let mut query = String::from("INSERT INTO StationHistory(Name,Url,Homepage,Favicon,Country,SubCountry,Language,Tags,IP,StationUuid,ChangeUuid) VALUES");
        let mut i = 0;
        for station in stations {
            if i > 0 {
                query.push_str(",");
            }
            query.push_str(&format!("(:name{i},:url{i},:homepage{i},:favicon{i},:country{i},:state{i},:language{i},:tags{i},:ip{i},:stationuuid{i},:changeuuid{i})",i=i));
            params.push((format!("name{i}",i=i), Value::from(station.name.clone())));
            params.push((format!("url{i}",i=i), Value::from(station.url.clone())));
            params.push((format!("homepage{i}",i=i), Value::from(station.homepage.clone())));
            params.push((format!("favicon{i}",i=i), Value::from(station.favicon.clone())));

            params.push((format!("country{i}",i=i), Value::from(station.country.clone())));
            params.push((format!("state{i}",i=i), Value::from(station.state.clone())));
            params.push((format!("language{i}",i=i), Value::from(station.language.clone())));
            params.push((format!("tags{i}",i=i), Value::from(station.tags.clone())));

            params.push((format!("ip{i}",i=i), Value::from(station.ip.clone())));
            params.push((format!("stationuuid{i}",i=i), Value::from(station.stationuuid.clone())));
            params.push((format!("changeuuid{i}",i=i), Value::from(station.changeuuid.clone())));

            i = i+1;
        }
        self.pool.prep_exec(query, params)?;
        Ok(())
    }*/

    pub fn station_exists(&self, stationuuid: &str) -> Result<bool, Box<std::error::Error>> {
        let params = params!{
            "stationuuid" => stationuuid,
        };
        let result = self.pool.prep_exec("SELECT COUNT(*) FROM Station WHERE StationUuid=:stationuuid", params)?;
        let count = self.get_single_column_number_intern(result)?;
        Ok(count > 0)
    }

    pub fn stationchange_exists(&self, changeuuid: &str) -> Result<bool, Box<std::error::Error>> {
        let params = params!{
            "changeuuid" => changeuuid,
        };
        let result = self.pool.prep_exec("SELECT COUNT(*) FROM StationHistory WHERE ChangeUuid=:changeuuid", params)?;
        let count = self.get_single_column_number_intern(result)?;
        Ok(count > 0)
    }

    pub fn insert_station_by_change(&self, stationchange: StationHistoryCurrent) -> Result<(),Box<std::error::Error>> {
        //self.insert_station_change(&stationchange)?;
        let changeexists = self.stationchange_exists(&stationchange.changeuuid)?;
        if !changeexists {
            let exists = self.station_exists(&stationchange.stationuuid)?;
            if exists {
                self.update_station((&stationchange).into())?;
            }else{
                self.add_station((&stationchange).into())?;
            }
        }
        Ok(())
    }

    pub fn stationcheck_exists(&self, checkuuid: &str) -> Result<bool, Box<std::error::Error>> {
        let params = params!{
            "checkuuid" => checkuuid,
        };
        let result = self.pool.prep_exec("SELECT COUNT(*) FROM StationCheck WHERE CheckUuid=:checkuuid", params)?;
        let count = self.get_single_column_number_intern(result)?;
        Ok(count > 0)
    }

    pub fn insert_station_check(&self, stationcheck: StationCheck) -> Result<(),Box<std::error::Error>> {
        let checkexists = self.stationcheck_exists(&stationcheck.checkuuid)?;
        if !checkexists {
            let query = format!("INSERT INTO StationCheck(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) 
                        VALUES(:stationuuid, :checkuuid, :source, :codec, :bitrate, :hls, :checkok, :checktime, :urlcache)");
            let params = params!{
                "stationuuid" => stationcheck.stationuuid,
                "checkuuid" => stationcheck.checkuuid,
                "source" => stationcheck.source,
                "codec" => stationcheck.codec,
                "bitrate" => stationcheck.bitrate,
                "hls" => stationcheck.hls,
                "checkok" => stationcheck.ok,
                "checktime" => stationcheck.timestamp,
                "urlcache" => stationcheck.urlcache,
            };

            self.pool.prep_exec(query, params)?;
        }
        Ok(())
    }

    pub fn get_checks(&self, stationuuid: Option<String>, checkuuid: Option<String>, seconds: u32) -> Vec<StationCheck> {
        let where_seconds = if seconds > 0 {
            format!(
                "TIMESTAMPDIFF(SECOND,CheckTime,now())<{seconds}",
                seconds = seconds
            )
        } else {
            String::from("")
        };

        let results = match stationuuid {
            Some(uuid) => {
                let where_checkuuid_str = if checkuuid.is_some() {
                    " AND CheckTime>=(SELECT CheckTime FROM StationCheckHistory WHERE ChangeUuid=:checkuuid) AND ChangeUuid<>:checkuuid"
                } else {
                    ""
                };

                let query = format!("SELECT {columns} from StationCheckHistory WHERE StationUuid=? {where_checkuuid} {where_seconds} ORDER BY CheckTime", columns = Connection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str);
                self.pool.prep_exec(query, (uuid,))
            }
            None => {
                let where_checkuuid_str = if checkuuid.is_some() {
                    " AND CheckTime>=(SELECT CheckTime FROM StationCheck WHERE ChangeUuid=:checkuuid) AND ChangeUuid<>:checkuuid"
                } else {
                    ""
                };

                let query = format!("SELECT {columns} from StationCheck WHERE 1=1 {where_checkuuid} {where_seconds} ORDER BY CheckTime", columns = Connection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str);
                self.pool.prep_exec(query, ())
            }
        };

        self.get_checks_internal(results)
    }

    pub fn get_stations_by_all(
        &self,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Vec<Station> {
        let order = self.filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " WHERE LastCheckOK=TRUE"
        } else {
            ""
        };

        let query: String = format!("SELECT {columns} from Station {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}",
            columns = Connection::COLUMNS, order = order, reverse = reverse_string,
            hidebroken = hidebroken_string, offset = offset, limit = limit);
        let results = self.pool.prep_exec(query, ());
        self.get_stations(results)
    }

    pub fn get_pull_server_lastid(&self, server: &str) -> Option<String> {
        let query: String = format!("SELECT lastid FROM PullServers WHERE name=:name");
        let results = self.pool.prep_exec(query, params!{
            "name" => server
        });
        match results {
            Ok(results) => {
                for result in results {
                    if let Ok(mut result) = result {
                        let lastid = result.take_opt("lastid");
                        if let Some(lastid) = lastid {
                            if let Ok(lastid) = lastid {
                                return Some(lastid);
                            }
                        }
                    }
                };
                None
            },
            _ => None
        }
    }

    pub fn get_pull_server_lastcheckid(&self, server: &str) -> Option<String> {
        let query: String = format!("SELECT lastcheckid FROM PullServers WHERE name=:name");
        let results = self.pool.prep_exec(query, params!{
            "name" => server
        });
        match results {
            Ok(results) => {
                for result in results {
                    if let Ok(mut result) = result {
                        let lastcheckid = result.take_opt("lastcheckid");
                        if let Some(lastcheckid) = lastcheckid {
                            if let Ok(lastcheckid) = lastcheckid {
                                return Some(lastcheckid);
                            }
                        }
                    }
                };
                None
            },
            _ => None
        }
    }

    pub fn set_pull_server_lastid(&self, server: &str, lastid: &str) -> Result<(),Box<std::error::Error>> {
        let params = params!{
            "name" => server,
            "lastid" => lastid,
        };
        let query_update: String = format!("UPDATE PullServers SET lastid=:lastid WHERE name=:name");
        let results_update = self.pool.prep_exec(query_update, &params)?;
        if results_update.affected_rows() == 0 {
            let query_insert: String = format!("INSERT INTO PullServers(name, lastid) VALUES(:name,:lastid)");
            self.pool.prep_exec(query_insert, &params)?;
        }
        Ok(())
    }

    pub fn set_pull_server_lastcheckid(&self, server: &str, lastcheckid: &str) -> Result<(),Box<std::error::Error>> {
        let params = params!{
            "name" => server,
            "lastcheckid" => lastcheckid,
        };
        let query_update: String = format!("UPDATE PullServers SET lastcheckid=:lastcheckid WHERE name=:name");
        let results_update = self.pool.prep_exec(query_update, &params)?;
        if results_update.affected_rows() == 0 {
            let query_insert: String = format!("INSERT INTO PullServers(name, lastcheckid) VALUES(:name,:lastcheckid)");
            self.pool.prep_exec(query_insert, &params)?;
        }
        Ok(())
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

    pub fn get_stations_broken(&self, limit: u32) -> Vec<Station> {
        self.get_stations_query(format!(
            "SELECT {columns} from Station WHERE LastCheckOK=FALSE ORDER BY rand() LIMIT {limit}",
            columns = Connection::COLUMNS,
            limit = limit
        ))
    }

    pub fn get_stations_improvable(&self, limit: u32) -> Vec<Station> {
        self.get_stations_query(format!(r#"SELECT {columns} from Station WHERE LastCheckOK=TRUE AND (Tags="" OR Country="") ORDER BY RAND() LIMIT {limit}"#,columns = Connection::COLUMNS, limit = limit))
    }

    pub fn get_stations_deleted(&self, limit: u32, id_str: &str) -> Vec<Station> {
        let id = id_str.parse::<u32>();
        let results = match id {
            Ok(id_number) => {
                let query = format!("SELECT {columns} FROM Station st RIGHT JOIN StationHistory sth ON st.StationID=sth.StationID WHERE st.StationID IS NULL AND sth.StationID=? ORDER BY sth.Creation DESC' {limit}",columns = Connection::COLUMNS, limit = limit);
                self.pool.prep_exec(query, (id_number,))
            }
            _ => {
                let query = format!("SELECT {columns} FROM Station st RIGHT JOIN StationHistory sth ON st.StationID=sth.StationID WHERE st.StationID IS NULL AND sth.StationUuid=? ORDER BY sth.Creation DESC' {limit}",columns = Connection::COLUMNS, limit = limit);
                self.pool.prep_exec(query, (id_str,))
            }
        };
        self.get_stations(results)
    }

    pub fn increase_clicks(&self, ip: &str, station: &Station) -> bool {
        let query = format!(r#"SELECT * FROM StationClick WHERE StationID={id} AND IP="{ip}" AND TIME_TO_SEC(TIMEDIFF(Now(),ClickTimestamp))<24*60*60"#, id = station.id, ip = ip);
        let result = self.pool.prep_exec(query, ()).unwrap();

        for resultsingle in result {
            for _ in resultsingle {
                return false;
            }
        }

        let query2 = format!(
            r#"INSERT INTO StationClick(StationID,IP) VALUES({id},"{ip}")"#,
            id = station.id,
            ip = ip
        );
        let result2 = self.pool.prep_exec(query2, ()).unwrap();

        let query3 = format!(
            "UPDATE Station SET ClickTimestamp=NOW() WHERE StationID={id}",
            id = station.id
        );
        let result3 = self.pool.prep_exec(query3, ()).unwrap();

        if result2.affected_rows() == 1 && result3.affected_rows() == 1 {
            true
        } else {
            false
        }
    }

    pub fn vote_for_station(&self, ip: &str, station: Option<Station>) -> Result<String, String> {
        match station {
            Some(station) => {
                // delete ipcheck entries after 1 day minutes
                let query_1_delete = format!(r#"DELETE FROM IPVoteCheck WHERE TIME_TO_SEC(TIMEDIFF(Now(),VoteTimestamp))>24*60*60"#);
                let _result_1_delete = self.pool.prep_exec(query_1_delete, ()).unwrap();

                // was there a vote from the ip in the last 1 day?
                let query_2_vote_check = format!(
                    r#"SELECT StationID FROM IPVoteCheck WHERE StationID={id} AND IP="{ip}""#,
                    id = station.id,
                    ip = ip
                );
                let result_2_vote_check = self.pool.prep_exec(query_2_vote_check, ()).unwrap();
                for resultsingle in result_2_vote_check {
                    for _ in resultsingle {
                        // do not allow vote
                        return Err("you are voting for the same station too often".to_string());
                    }
                }

                // add vote entry
                let query_3_insert_votecheck = format!(
                    r#"INSERT INTO IPVoteCheck(IP,StationID) VALUES("{ip}",{id})"#,
                    id = station.id,
                    ip = ip
                );
                let result_3_insert_votecheck =
                    self.pool.prep_exec(query_3_insert_votecheck, ()).unwrap();
                if result_3_insert_votecheck.affected_rows() == 0 {
                    return Err("could not insert vote check".to_string());
                }

                // vote for station
                let query_4_update_votes = format!(
                    "UPDATE Station SET Votes=Votes+1 WHERE StationID={id}",
                    id = station.id
                );
                let result_4_update_votes = self.pool.prep_exec(query_4_update_votes, ()).unwrap();
                if result_4_update_votes.affected_rows() == 1 {
                    Ok("voted for station successfully".to_string())
                } else {
                    Err("could not find station with matching id".to_string())
                }
            }
            _ => Err("could not find station with matching id".to_string()),
        }
    }

    pub fn get_stations_advanced(
        &self,
        name: Option<String>,
        name_exact: bool,
        country: Option<String>,
        country_exact: bool,
        state: Option<String>,
        state_exact: bool,
        language: Option<String>,
        language_exact: bool,
        tag: Option<String>,
        tag_exact: bool,
        tag_list: Vec<String>,
        bitrate_min: u32,
        bitrate_max: u32,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Vec<Station> {
        let order = self.filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let mut query = format!(
            "SELECT {columns} from Station WHERE",
            columns = Connection::COLUMNS
        );
        query.push_str(" Bitrate >= :bitrate_min AND Bitrate <= :bitrate_max");
        if name.is_some() {
            if name_exact {
                query.push_str(" AND Name=:name");
            } else {
                query.push_str(" AND Name LIKE CONCAT('%',:name,'%')");
            }
        }
        if country.is_some() {
            if country_exact {
                query.push_str(" AND Country=:country");
            } else {
                query.push_str(" AND Country LIKE CONCAT('%',:country,'%')");
            }
        }
        if state.is_some() {
            if state_exact {
                query.push_str(" AND Subcountry=:state");
            } else {
                query.push_str(" AND Subcountry LIKE CONCAT('%',:state,'%')");
            }
        }
        if language.is_some() {
            if language_exact {
                query.push_str(" AND ( Language=:language OR Language LIKE CONCAT('%,',:language,',%') OR Language LIKE CONCAT('%,',:language) OR Language LIKE CONCAT(:language,',%'))");
            } else {
                query.push_str(" AND Language LIKE CONCAT('%',:language,'%')");
            }
        }
        if tag.is_some() {
            if tag_exact {
                query.push_str(" AND ( Tags=:tag OR Tags LIKE CONCAT('%,',:tag,',%') OR Tags LIKE CONCAT('%,',:tag) OR Tags LIKE CONCAT(:tag,',%'))");
            } else {
                query.push_str(" AND Tags LIKE CONCAT('%',:tag,'%')");
            }
        }
        let mut params = params!{
            "name" => name.unwrap_or_default(),
            "country" => country.unwrap_or_default(),
            "state" => state.unwrap_or_default(),
            "language" => language.unwrap_or_default(),
            "tag" => tag.unwrap_or_default(),
            "bitrate_min" => bitrate_min,
            "bitrate_max" => bitrate_max,
        };
        let mut i = 0;
        for tag in tag_list {
            if tag_exact {
                query.push_str(&format!(" AND ( Tags=:tag{i} OR Tags LIKE CONCAT('%,',:tag{i},',%') OR Tags LIKE CONCAT('%,',:tag{i}) OR Tags LIKE CONCAT(:tag{i},',%'))",i=i));
            } else {
                query.push_str(&format!(" AND Tags LIKE CONCAT('%',:tag{i},'%')",i=i));
            }
            params.push((format!("tag{i}",i=i), Value::from(tag)));
            i += 1;
        }
        query.push_str(&format!(
            " {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}",
            order = order,
            reverse = reverse_string,
            hidebroken = hidebroken_string,
            offset = offset,
            limit = limit
        ));
        
        let results = self.pool.prep_exec(
            query,
            params,
        );
        self.get_stations(results)
    }

    pub fn get_stations_deleted_all(&self, limit: u32) -> Vec<Station> {
        self.get_stations_query(format!("SELECT {columns} FROM Station st RIGHT JOIN StationHistory sth ON st.StationID=sth.StationID WHERE st.StationID IS NULL ORDER BY sth.Creation DESC' {limit}",columns = Connection::COLUMNS, limit = limit))
    }

    pub fn get_stations_by_column(
        &self,
        column_name: &str,
        search: String,
        exact: bool,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Vec<Station> {
        let order = self.filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let query: String = if exact {
            format!("SELECT {columns} from Station WHERE {column_name}=? {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = Connection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        } else {
            format!("SELECT {columns} from Station WHERE {column_name} LIKE CONCAT('%',?,'%') {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = Connection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        };
        let results = self.pool.prep_exec(query, (search,));
        self.get_stations(results)
    }

    pub fn get_stations_by_column_multiple(
        &self,
        column_name: &str,
        search: Option<String>,
        exact: bool,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Vec<Station> {
        let order = self.filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let query: String = if exact {
            format!(
                r"SELECT {columns} from Station WHERE ({column_name}=?
             OR {column_name} LIKE CONCAT('%,',?,',%')
             OR {column_name} LIKE CONCAT(?,',%')
             OR {column_name} LIKE CONCAT('%,',?))
             {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}",
                columns = Connection::COLUMNS,
                order = order,
                reverse = reverse_string,
                hidebroken = hidebroken_string,
                offset = offset,
                limit = limit,
                column_name = column_name
            )
        } else {
            format!("SELECT {columns} from Station WHERE {column_name} LIKE CONCAT('%',?,'%') {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = Connection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        };
        let results = if exact {
            self.pool
                .prep_exec(query, (&search, &search, &search, &search))
        } else {
            self.pool.prep_exec(query, (search,))
        };
        self.get_stations(results)
    }

    pub fn get_station_by_id_or_uuid(&self, id_str: &str) -> Option<Station> {
        let id = id_str.parse::<u32>();
        let results = match id {
            Ok(id_number) => {
                let query = format!(
                    "SELECT {columns} from Station WHERE StationID=? ORDER BY Name",
                    columns = Connection::COLUMNS
                );
                self.pool.prep_exec(query, (id_number,))
            }
            _ => {
                let query = format!(
                    "SELECT {columns} from Station WHERE StationUuid=? ORDER BY Name",
                    columns = Connection::COLUMNS
                );
                self.pool.prep_exec(query, (id_str,))
            }
        };
        let mut stations = self.get_stations(results);
        if stations.len() == 1 {
            Some(stations.pop().unwrap())
        } else {
            None
        }
    }

    pub fn get_stations_by_id(&self, id: i32) -> Vec<Station> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station WHERE StationID={id} ORDER BY Name",
            columns = Connection::COLUMNS,
            id = id
        );
        self.get_stations_query(query)
    }

    pub fn get_stations_topvote(&self, limit: u32) -> Vec<Station> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY Votes DESC LIMIT {limit}",
            columns = Connection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    pub fn get_stations_topclick(&self, limit: u32) -> Vec<Station> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY clickcount DESC LIMIT {limit}",
            columns = Connection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    pub fn get_stations_lastclick(&self, limit: u32) -> Vec<Station> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY ClickTimestamp DESC LIMIT {limit}",
            columns = Connection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    pub fn get_stations_lastchange(&self, limit: u32) -> Vec<Station> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY Creation DESC LIMIT {limit}",
            columns = Connection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    pub fn get_changes(&self, stationuuid: Option<String>, changeuuid: Option<String>) -> Vec<StationHistoryCurrent> {
        let changeuuid_str = if changeuuid.is_some() {
            " AND Creation>=(SELECT Creation FROM StationHistory WHERE ChangeUuid=:changeuuid) AND ChangeUuid<>:changeuuid"
        } else {
            ""
        };

        let stationuuid_str = if stationuuid.is_some() {
            " AND StationUuid=:stationuuid"
        }else{
            ""
        };
        
        let query: String = format!("SELECT StationID,ChangeUuid,
                StationUuid,Name,
                Url,Homepage,
                Favicon,Tags,
                Country,Subcountry,
                Language,Votes,
                NegativeVotes,Creation,Ip from StationHistory WHERE 1=:mynumber {changeuuid_str} {stationuuid} ORDER BY Creation ASC", changeuuid_str = changeuuid_str, stationuuid = stationuuid_str);
        let results = self.pool.prep_exec(query, params! {
            "mynumber" => 1,
            "stationuuid" => stationuuid.unwrap_or(String::from("")),
            "changeuuid" => changeuuid.unwrap_or(String::from(""))
        });
        self.get_stations_history(results)
    }

    fn get_stations_query(&self, query: String) -> Vec<Station> {
        let results = self.pool.prep_exec(query, ());
        self.get_stations(results)
    }

    fn get_stations(&self, results: ::mysql::Result<QueryResult<'static>>) -> Vec<Station> {
        let mut stations: Vec<Station> = vec![];
        for result in results {
            for row_ in result {
                let mut row = row_.unwrap();
                let s = Station::new(
                    row.take("StationID").unwrap(),
                    row.take("ChangeUuid").unwrap_or("".to_string()),
                    row.take("StationUuid").unwrap_or("".to_string()),
                    row.take("Name").unwrap_or("".to_string()),
                    row.take("Url").unwrap_or("".to_string()),
                    row.take("UrlCache").unwrap_or("".to_string()),
                    row
                        .take_opt("Homepage")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Favicon")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Tags")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Country")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Subcountry")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Language")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row.take_opt("Votes").unwrap_or(Ok(0)).unwrap_or(0),
                    row.take_opt("NegativeVotes").unwrap_or(Ok(0)).unwrap_or(0),
                    row
                        .take_opt("CreationFormated")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Ip")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Codec")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
                    row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0),
                    row.take_opt("LastCheckOK").unwrap_or(Ok(0)).unwrap_or(0),
                    row
                        .take_opt("LastCheckTimeFormated")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("LastCheckOkTimeFormated")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("ClickTimestampFormated")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row.take_opt("clickcount").unwrap_or(Ok(0)).unwrap_or(0),
                    row.take_opt("ClickTrend").unwrap_or(Ok(0)).unwrap_or(0),
                );
                stations.push(s);
            }
        }

        stations
    }

    fn get_stations_history(&self, results: ::mysql::Result<QueryResult<'static>>) -> Vec<StationHistoryCurrent> {
        let mut changes: Vec<StationHistoryCurrent> = vec![];
        for result in results {
            for row_ in result {
                let mut row = row_.unwrap();
                let s = StationHistoryCurrent::new(
                    row.take("StationID").unwrap(),
                    row.take("ChangeUuid").unwrap_or("".to_string()),
                    row.take("StationUuid").unwrap_or("".to_string()),
                    row.take("Name").unwrap_or("".to_string()),
                    row.take("Url").unwrap_or("".to_string()),
                    row
                        .take_opt("Homepage")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Favicon")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Tags")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Country")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Subcountry")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Language")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row.take_opt("Votes").unwrap_or(Ok(0)).unwrap_or(0),
                    row.take_opt("NegativeVotes").unwrap_or(Ok(0)).unwrap_or(0),
                    row
                        .take_opt("Creation")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("Ip")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                );
                changes.push(s);
            }
        }

        changes
    }

    fn get_checks_internal(
        &self,
        results: ::mysql::Result<QueryResult<'static>>,
    ) -> Vec<StationCheck> {
        let mut checks: Vec<StationCheck> = vec![];
        for result in results {
            for row_ in result {
                let mut row = row_.unwrap();
                let s = StationCheck::new(
                    row.take("CheckID").unwrap(),
                    row.take("StationUuid").unwrap_or("".to_string()),
                    row.take("CheckUuid").unwrap_or("".to_string()),
                    row.take("Source").unwrap_or("".to_string()),
                    row
                        .take_opt("Codec")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
                    row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0),
                    row.take_opt("CheckOK").unwrap_or(Ok(0)).unwrap_or(0),
                    row
                        .take_opt("CheckTimeFormated")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                    row
                        .take_opt("UrlCache")
                        .unwrap_or(Ok("".to_string()))
                        .unwrap_or("".to_string()),
                );
                checks.push(s);
            }
        }

        checks
    }

    pub fn get_1_n(
        &self,
        column: &str,
        search: Option<String>,
        order: String,
        reverse: bool,
        hidebroken: bool,
    ) -> Vec<Result1n> {
        let query: String;
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let result = match search {
            Some(value) => {
                query = format!("SELECT {column} AS name,COUNT(*) AS stationcount FROM Station WHERE {column} LIKE CONCAT('%',?,'%') AND {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
                self.pool.prep_exec(query, (value,))
            }
            None => {
                query = format!("SELECT {column} AS name,COUNT(*) AS stationcount FROM Station WHERE {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
                self.pool.prep_exec(query, ())
            }
        };

        let stations: Vec<Result1n> = result
            .map(|result| {
                result
                    .map(|x| x.unwrap())
                    .map(|row| {
                        let (name, stationcount) = mysql::from_row(row);
                        Result1n::new(name, stationcount)
                    }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap(); // Unwrap `Vec<Payment>`
        stations
    }

    pub fn get_states(
        &self,
        country: Option<String>,
        search: Option<String>,
        order: String,
        reverse: bool,
        hidebroken: bool,
    ) -> Vec<State> {
        let mut params: Vec<Value> = Vec::with_capacity(1);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let country_string = match country {
            Some(c) => {
                params.push(c.into());
                format!(" AND Country=?")
            }
            None => "".to_string(),
        };
        let search_string = match search {
            Some(c) => {
                params.push((format!("%{}%", c)).into());
                format!(" AND Subcountry LIKE ?")
            }
            None => "".to_string(),
        };

        let mut my_stmt = self.pool.prepare(format!(r"SELECT Subcountry AS name,Country,COUNT(*) AS stationcount FROM Station WHERE Subcountry <> '' {country} {search} {hidebroken} GROUP BY Subcountry, Country ORDER BY {order} {reverse}",hidebroken = hidebroken_string, order = order, country = country_string, reverse = reverse_string, search = search_string)).unwrap();
        let my_results = my_stmt.execute(params);
        let mut states: Vec<State> = vec![];

        for my_result in my_results {
            for my_row in my_result {
                let mut row_unwrapped = my_row.unwrap();
                states.push(State::new(
                    row_unwrapped.take(0).unwrap_or("".into()),
                    row_unwrapped.take(1).unwrap_or("".into()),
                    row_unwrapped.take(2).unwrap_or(0),
                ));
            }
        }
        states
    }

    pub fn get_extra(
        &self,
        table_name: &str,
        column_name: &str,
        search: Option<String>,
        order: String,
        reverse: bool,
        hidebroken: bool,
    ) -> Vec<ExtraInfo> {
        let mut params: Vec<Value> = Vec::with_capacity(1);
        let mut items = vec![];
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let search_string = match search {
            Some(c) => {
                params.push((format!("%{}%", c)).into());
                format!(" AND {} LIKE ?", column_name)
            }
            None => "".to_string(),
        };
        let mut stmt = self.pool.prepare(format!("SELECT {column_name} AS name, StationCount as stationcount, StationCountWorking FROM {table_name} WHERE {column_name} <> '' {search} {hidebroken} ORDER BY {order} {reverse}",search = search_string, order = order, reverse = reverse_string, hidebroken = hidebroken_string, table_name = table_name, column_name = column_name)).unwrap();
        let my_results = stmt.execute(params);
        for my_result in my_results {
            for my_row in my_result {
                let mut row_unwrapped = my_row.unwrap();
                items.push(ExtraInfo::new(
                    row_unwrapped.take(0).unwrap_or("".into()),
                    row_unwrapped.take(1).unwrap_or(0),
                    row_unwrapped.take(2).unwrap_or(0),
                ));
            }
        }
        items
    }
}

fn get_cached_items(
    pool: &mysql::Pool,
    table_name: &str,
    column_name: &str,
) -> HashMap<String, u32> {
    let mut items = HashMap::new();
    let mut my_stmt = pool
        .prepare(format!(
            "SELECT {column_name},StationCount FROM {table_name}",
            table_name = table_name,
            column_name = column_name
        )).unwrap();
    let my_results = my_stmt.execute(());

    for my_result in my_results {
        for my_row in my_result {
            let mut row_unwrapped = my_row.unwrap();
            let key: String = row_unwrapped.take(0).unwrap_or("".into());
            let value: u32 = row_unwrapped.take(1).unwrap_or(0);
            let lower = key.to_lowercase();
            items.insert(lower, value);
        }
    }
    items
}

fn get_stations_multi_items(pool: &mysql::Pool, column_name: &str) -> HashMap<String, u32> {
    let mut items = HashMap::new();
    let mut my_stmt = pool
        .prepare(format!(
            "SELECT {column_name} FROM Station",
            column_name = column_name
        )).unwrap();
    let my_results = my_stmt.execute(());

    for my_result in my_results {
        for my_row in my_result {
            let mut row_unwrapped = my_row.unwrap();
            let tags_str: String = row_unwrapped.take(0).unwrap_or("".into());
            let tags_arr = tags_str.split(',');
            for single_tag in tags_arr {
                let single_tag_trimmed = single_tag.trim().to_lowercase();
                if single_tag_trimmed != "" {
                    let counter = items.entry(single_tag_trimmed).or_insert(0);
                    *counter += 1;
                }
            }
        }
    }
    items
}

fn update_cache_item(
    pool: &mysql::Pool,
    tag: &String,
    count: u32,
    table_name: &str,
    column_name: &str,
) {
    let mut my_stmt = pool
        .prepare(format!(
            r"UPDATE {table_name} SET StationCount=? WHERE {column_name}=?",
            table_name = table_name,
            column_name = column_name
        )).unwrap();
    let params = (count, tag);
    let result = my_stmt.execute(params);
    match result {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
        }
    }
}

fn insert_to_cache(
    pool: &mysql::Pool,
    tags: HashMap<&String, u32>,
    table_name: &str,
    column_name: &str,
) {
    let query = format!(
        "INSERT INTO {table_name}({column_name},StationCount) VALUES(?,?)",
        table_name = table_name,
        column_name = column_name
    );
    let mut my_stmt = pool.prepare(query.trim_matches(',')).unwrap();
    for item in tags.iter() {
        let result = my_stmt.execute((item.0, item.1));
        match result {
            Ok(_) => {}
            Err(err) => {
                error!("{}", err);
            }
        }
    }
}

fn remove_from_cache(pool: &mysql::Pool, tags: Vec<&String>, table_name: &str, column_name: &str) {
    let mut query = format!(
        "DELETE FROM {table_name} WHERE {column_name}=''",
        table_name = table_name,
        column_name = column_name
    );
    for _ in 0..tags.len() {
        query.push_str(" OR ");
        query.push_str(column_name);
        query.push_str("=?");
    }
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(tags);
    match result {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
        }
    }
}

pub struct RefreshCacheStatus{
    old_items: usize,
    new_items: usize,
    changed_items: usize,
}

pub fn refresh_cache_items(
    pool: &mysql::Pool,
    cache_table_name: &str,
    cache_column_name: &str,
    station_column_name: &str,
)-> RefreshCacheStatus {
    let items_cached = get_cached_items(pool, cache_table_name, cache_column_name);
    let items_current = get_stations_multi_items(pool, station_column_name);
    let mut changed = 0;

    let mut to_delete = vec![];
    for item_cached in items_cached.keys() {
        if !items_current.contains_key(item_cached) {
            to_delete.push(item_cached);
        }
    }
    remove_from_cache(pool, to_delete, cache_table_name, cache_column_name);

    let mut to_insert: HashMap<&String, u32> = HashMap::new();
    for item_current in items_current.keys() {
        if !items_cached.contains_key(item_current) {
            //self.insert_tag(tag_current, *tags_current.get(tag_current).unwrap_or(&0));
            to_insert.insert(item_current, *items_current.get(item_current).unwrap_or(&0));
        } else {
            let value_new = *items_current.get(item_current).unwrap_or(&0);
            let value_old = *items_cached.get(item_current).unwrap_or(&0);
            if value_old != value_new {
                update_cache_item(
                    pool,
                    item_current,
                    value_new,
                    cache_table_name,
                    cache_column_name,
                );
                changed = changed + 1;
            }
        }
    }
    insert_to_cache(pool, to_insert, cache_table_name, cache_column_name);
    trace!(
        "{}: {} -> {}, Changed: {}",
        station_column_name,
        items_cached.len(),
        items_current.len(),
        changed
    );
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
    RefreshCacheStatus{
        old_items: items_cached.len(),
        new_items: items_current.len(),
        changed_items: changed,
    }
}

fn start_refresh_worker(connection_string: String, update_caches_interval: u64) {
    thread::spawn(move || {
        loop {
            let pool = mysql::Pool::new(&connection_string);
            match pool {
                Ok(p) => {
                    trace!("REFRESH START");
                    let tags = refresh_cache_items(&p, "TagCache", "TagName", "Tags");
                    let languages = refresh_cache_items(&p, "LanguageCache", "LanguageName", "Language");
                    debug!("Refresh(Tags={}->{}, Languages={}->{})", tags.old_items, tags.new_items, languages.old_items, languages.new_items);
                }
                Err(e) => error!("{}", e),
            }

            thread::sleep(::std::time::Duration::new(update_caches_interval, 0));
        }
    });
}

pub fn new(connection_string: &String, update_caches_interval: u64, ignore_migration_errors: bool, allow_database_downgrade: bool) -> Result<Connection, Box<std::error::Error>> {
    let connection_string2 = connection_string.clone();
    let mut migrations = Migrations::new(connection_string);
    migrations.add_migration("20190104_014300_CreateStation",
r#"CREATE TABLE `Station` (
  `StationID` int(11) NOT NULL AUTO_INCREMENT,
  `Name` text,
  `Url` text,
  `Homepage` text,
  `Favicon` text,
  `Creation` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `Country` varchar(50) DEFAULT NULL,
  `Language` varchar(50) DEFAULT NULL,
  `Tags` text,
  `Votes` int(11) DEFAULT '0',
  `NegativeVotes` int(11) NOT NULL DEFAULT '0',
  `Source` varchar(20) DEFAULT NULL,
  `Subcountry` varchar(50) DEFAULT NULL,
  `clickcount` int(11) DEFAULT '0',
  `ClickTrend` int(11) DEFAULT '0',
  `ClickTimestamp` datetime DEFAULT NULL,
  `Codec` varchar(20) DEFAULT NULL,
  `LastCheckOK` tinyint(1) NOT NULL DEFAULT '1',
  `LastCheckTime` datetime DEFAULT NULL,
  `Bitrate` int(11) NOT NULL DEFAULT '0',
  `UrlCache` text NOT NULL,
  `LastCheckOkTime` datetime DEFAULT NULL,
  `Hls` tinyint(1) NOT NULL DEFAULT '0',
  `IP` varchar(50) NOT NULL DEFAULT '',
  `ChangeUuid` char(36) DEFAULT NULL,
  `StationUuid` char(36) DEFAULT NULL,
  PRIMARY KEY (`StationID`),
  UNIQUE KEY `ChangeUuid` (`ChangeUuid`),
  UNIQUE KEY `StationUuid` (`StationUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#, "DROP TABLE Station;");

    migrations.add_migration("20190104_014301_CreateIPVoteCheck",
r#"CREATE TABLE `IPVoteCheck` (
  `CheckID` int(11) NOT NULL AUTO_INCREMENT,
  `IP` varchar(15) NOT NULL,
  `StationID` int(11) NOT NULL,
  `VoteTimestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`CheckID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,"DROP TABLE IPVoteCheck");

    migrations.add_migration("20190104_014302_CreateLanguageCache",
r#"CREATE TABLE `LanguageCache` (
  `LanguageName` varchar(500) COLLATE utf8mb4_bin NOT NULL,
  `StationCount` int(11) DEFAULT '0',
  `StationCountWorking` int(11) DEFAULT '0',
  PRIMARY KEY (`LanguageName`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE LanguageCache");

    migrations.add_migration("20190104_014303_CreateTagCache",
r#"CREATE TABLE `TagCache` (
  `TagName` varchar(500) COLLATE utf8mb4_bin NOT NULL,
  `StationCount` int(11) DEFAULT '0',
  `StationCountWorking` int(11) DEFAULT '0',
  PRIMARY KEY (`TagName`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;"#,"DROP TABLE TagCache");

    migrations.add_migration("20190104_014304_CreateStationCheck",
r#"CREATE TABLE `StationCheck` (
  `CheckID` int(11) NOT NULL AUTO_INCREMENT,
  `StationUuid` char(36) NOT NULL,
  `CheckUuid` char(36) NOT NULL,
  `Source` varchar(100) NOT NULL,
  `Codec` varchar(20) DEFAULT NULL,
  `Bitrate` int(11) NOT NULL DEFAULT '0',
  `Hls` tinyint(1) NOT NULL DEFAULT '0',
  `CheckOK` tinyint(1) NOT NULL DEFAULT '1',
  `CheckTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `UrlCache` text,
  PRIMARY KEY (`CheckID`),
  UNIQUE KEY `CheckUuid` (`CheckUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,"DROP TABLE StationCheck");

    migrations.add_migration("20190104_014305_CreateStationClick",
r#"CREATE TABLE `StationClick` (
  `ClickID` int(11) NOT NULL AUTO_INCREMENT,
  `StationID` int(11) DEFAULT NULL,
  `ClickTimestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `IP` varchar(50) DEFAULT NULL,
  PRIMARY KEY (`ClickID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,"DROP TABLE StationClick");

    migrations.add_migration("20190104_014306_CreateStationHistory",
r#"CREATE TABLE `StationHistory` (
  `StationChangeID` int(11) NOT NULL AUTO_INCREMENT,
  `StationID` int(11) NOT NULL,
  `Name` text,
  `Url` text,
  `Homepage` text,
  `Favicon` text,
  `Creation` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `Country` varchar(50) DEFAULT NULL,
  `Subcountry` varchar(50) DEFAULT NULL,
  `Language` varchar(50) DEFAULT NULL,
  `Tags` text,
  `Votes` int(11) DEFAULT '0',
  `NegativeVotes` int(11) DEFAULT '0',
  `IP` varchar(50) NOT NULL DEFAULT '',
  `ChangeUuid` char(36) DEFAULT NULL,
  `StationUuid` char(36) DEFAULT NULL,
  PRIMARY KEY (`StationChangeID`),
  UNIQUE KEY `ChangeUuid` (`ChangeUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,"DROP TABLE StationHistory");

    migrations.add_migration("20190104_014307_CreatePullServers",
r#"CREATE TABLE PullServers (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name TEXT NOT NULL,
    lastid TEXT,
    lastcheckid TEXT
);"#, "DROP TABLE PullServers;");

    migrations.add_migration("20190104_014304_CreateStationCheckHistory",
r#"CREATE TABLE `StationCheckHistory` (
  `CheckID` int(11) NOT NULL AUTO_INCREMENT,
  `StationUuid` char(36) NOT NULL,
  `CheckUuid` char(36) NOT NULL,
  `Source` varchar(100) NOT NULL,
  `Codec` varchar(20) DEFAULT NULL,
  `Bitrate` int(11) NOT NULL DEFAULT '0',
  `Hls` tinyint(1) NOT NULL DEFAULT '0',
  `CheckOK` tinyint(1) NOT NULL DEFAULT '1',
  `CheckTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `UrlCache` text,
  PRIMARY KEY (`CheckID`),
  UNIQUE KEY `CheckUuid` (`CheckUuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,"DROP TABLE StationCheckHistory");

    migrations.do_migrations(ignore_migration_errors, allow_database_downgrade)?;

    let pool = mysql::Pool::new(connection_string);
    match pool {
        Ok(p) => {
            let c = Connection { pool: p };

            if update_caches_interval > 0 {
                start_refresh_worker(connection_string2, update_caches_interval);
            }

            Ok(c)
        }
        Err(e) => Err(Box::new(api_error::ApiError::ConnectionError(e.to_string()))),
    }
}
