use crate::api::data::Station;
use crate::db::models::StationClickItemNew;
use crate::db::models::State;
use crate::db::models::ExtraInfo;
use crate::db::models::StationItem;
use crate::db::models::StationCheckItem;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationChangeItemNew;
use crate::db::models::StationHistoryItem;
use crate::db::models::StationClickItem;
use crate::db::MysqlConnection;
use crate::db::DbError;
use std::error::Error;
use std::collections::HashSet;
use std::collections::HashMap;

pub trait DbConnection {
    fn get_station_count_broken(&self) -> Result<u64, Box<dyn Error>>;
    fn get_station_count_working(&self) -> Result<u64, Box<dyn Error>>;
    fn get_station_count_todo(&self, hours: u32) -> Result<u64, Box<dyn Error>>;
    fn get_deletable_never_working(&self, seconds: u64) -> Result<u64, Box<dyn Error>>;
    fn get_deletable_were_working(&self, seconds: u64) -> Result<u64, Box<dyn Error>>;
    fn get_tag_count(&self) -> Result<u64, Box<dyn Error>>;
    fn get_country_count(&self) -> Result<u64, Box<dyn Error>>;
    fn get_language_count(&self) -> Result<u64, Box<dyn Error>>;
    fn get_click_count_last_hour(&self) -> Result<u64, Box<dyn Error>>;
    fn get_click_count_last_day(&self) -> Result<u64, Box<dyn Error>>;
    fn get_stations_to_check(&mut self, hours: u32, itemcount: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_station_by_uuid(&self, id_str: &str) -> Result<Vec<StationItem>,Box<dyn Error>>;
    fn get_stations_by_column_multiple(&self,column_name: &str,search: Option<String>,exact: bool,order: &str,reverse: bool,hidebroken: bool,offset: u32,limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_by_all(&self,order: &str,reverse: bool,hidebroken: bool,offset: u32,limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_advanced(
        &self,name: Option<String>,name_exact: bool,country: Option<String>,country_exact: bool,countrycode: Option<String>,
        state: Option<String>,state_exact: bool,language: Option<String>,
        language_exact: bool,tag: Option<String>,tag_exact: bool,tag_list: Vec<String>,
        bitrate_min: u32,bitrate_max: u32,order: &str,reverse: bool,hidebroken: bool,offset: u32,limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_changes(&self, stationuuid: Option<String>, changeuuid: Option<String>) -> Result<Vec<StationHistoryItem>, Box<dyn Error>>;
    
    fn add_station_opt(&self, name: Option<String>, url: Option<String>, homepage: Option<String>, favicon: Option<String>,
        country: Option<String>, countrycode: Option<String>, state: Option<String>, language: Option<String>, tags: Option<String>) -> Result<String, Box<dyn Error>>;

    fn get_stations_broken(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_improvable(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_topvote(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_topclick(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_lastclick(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_lastchange(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;
    fn get_stations_by_column(&self,column_name: &str,search: String,exact: bool,order: &str,reverse: bool,hidebroken: bool,offset: u32,limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;

    fn get_pull_server_lastid(&self, server: &str) -> Option<String>;
    fn set_pull_server_lastid(&self, server: &str, lastid: &str) -> Result<(),Box<dyn std::error::Error>>;
    fn get_pull_server_lastcheckid(&self, server: &str) -> Option<String>;
    fn set_pull_server_lastcheckid(&self, server: &str, lastcheckid: &str) -> Result<(),Box<dyn std::error::Error>>;
    fn get_pull_server_lastclickid(&self, server: &str) -> Option<String>;
    fn set_pull_server_lastclickid(&self, server: &str, lastclickuuid: &str) -> Result<(),Box<dyn std::error::Error>>;

    fn insert_station_by_change(&self, list_station_changes: &Vec<StationChangeItemNew>) -> Result<Vec<String>,Box<dyn std::error::Error>>;

    fn get_extra(&self, table_name: &str, column_name: &str, search: Option<String>, order: String, reverse: bool, hidebroken: bool) -> Result<Vec<ExtraInfo>, Box<dyn Error>>;
    fn get_1_n(&self, column: &str, search: Option<String>, order: String, reverse: bool, hidebroken: bool) -> Result<Vec<ExtraInfo>, Box<dyn Error>>;
    fn get_states(&self, country: Option<String>, search: Option<String>, order: String, reverse: bool, hidebroken: bool) -> Result<Vec<State>, Box<dyn Error>>;
    fn get_checks(&self, stationuuid: Option<String>, checkuuid: Option<String>, seconds: u32, include_history: bool) -> Result<Vec<StationCheckItem>, Box<dyn Error>>;
    fn get_clicks(&self, stationuuid: Option<String>, clickuuid: Option<String>, seconds: u32) -> Result<Vec<StationClickItem>, Box<dyn Error>>;

    fn insert_checks(&self, list: &Vec<StationCheckItemNew>) -> Result<HashSet<String>, Box<dyn std::error::Error>>;
    fn update_station_with_check_data(&self, list: &Vec<StationCheckItemNew>, local: bool) -> Result<(), Box<dyn Error>>;

    fn insert_clicks(&self, list: &Vec<StationClickItemNew>) -> Result<(), Box<dyn Error>>;

    fn delete_never_working(&mut self, seconds: u64) -> Result<(), Box<dyn Error>>;
    fn delete_were_working(&mut self, seconds: u64) -> Result<(), Box<dyn Error>>;
    fn delete_old_checks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>>;
    fn delete_old_clicks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>>;
    fn remove_unused_ip_infos_from_stationclicks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>>;
    fn remove_illegal_icon_links(&mut self) -> Result<(), Box<dyn Error>>;
    
    fn update_stations_clickcount(&self) -> Result<(), Box<dyn Error>>;

    fn get_stations_multi_items(&self, column_name: &str) -> Result<HashMap<String, (u32,u32)>, Box<dyn Error>>;
    fn get_cached_items(&self, table_name: &str, column_name: &str) -> Result<HashMap<String, (u32, u32)>, Box<dyn Error>>;
    fn update_cache_item(&self, tag: &String, count: u32, count_working: u32, table_name: &str, column_name: &str) -> Result<(), Box<dyn Error>>;
    fn insert_to_cache(&self, tags: HashMap<&String, (u32,u32)>, table_name: &str, column_name: &str) -> Result<(), Box<dyn Error>>;
    fn remove_from_cache(&self, tags: Vec<&String>, table_name: &str, column_name: &str) -> Result<(), Box<dyn Error>>;

    fn vote_for_station(&self, ip: &str, station: Option<StationItem>) -> Result<String, Box<dyn Error>>;
    fn increase_clicks(&self, ip: &str, station: &StationItem, seconds: u64) -> Result<bool,Box<dyn Error>>;
    fn sync_votes(&self, list: Vec<Station>) -> Result<(), Box<dyn Error>>;
}

pub fn connect(connection_string: String) -> Result<Box<dyn DbConnection>, Box<dyn std::error::Error>> {
    if connection_string.starts_with("mysql://") {
        return Ok(Box::new(MysqlConnection::new(&connection_string)?));
    }else{
        return Err(Box::new(DbError::ConnectionError(String::from("Unknown protocol for database connection"))));
    }
}