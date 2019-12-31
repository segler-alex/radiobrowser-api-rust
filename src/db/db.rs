use crate::db::models::State;
use crate::db::models::ExtraInfo;
use crate::db::models::StationItem;
use crate::db::models::StationCheckItem;
use crate::db::models::StationCheckItemNew;
use std::error::Error;

pub trait DbConnection {
    fn get_station_count_broken(&self) -> Result<u64, Box<dyn Error>>;
    fn get_station_count_working(&self) -> Result<u64, Box<dyn Error>>;
    fn get_station_count_todo(&self, hours: u32) -> Result<u64, Box<dyn Error>>;
    fn get_checks_todo_count(&self, hours: u32, source: &str) -> Result<u64, Box<dyn Error>>;
    fn get_deletable_never_working(&self, hours: u32) -> Result<u64, Box<dyn Error>>;
    fn get_deletable_were_working(&self, hours: u32) -> Result<u64, Box<dyn Error>>;
    fn get_tag_count(&self) -> Result<u64, Box<dyn Error>>;
    fn get_country_count(&self) -> Result<u64, Box<dyn Error>>;
    fn get_language_count(&self) -> Result<u64, Box<dyn Error>>;
    fn get_click_count_last_hour(&self) -> Result<u64, Box<dyn Error>>;
    fn get_click_count_last_day(&self) -> Result<u64, Box<dyn Error>>;
    fn get_stations_to_check(&mut self, hours: u32, itemcount: u32) -> Result<Vec<StationItem>, Box<dyn Error>>;

    fn get_pull_server_lastid(&self, server: &str) -> Option<String>;
    fn set_pull_server_lastid(&self, server: &str, lastid: &str) -> Result<(),Box<dyn std::error::Error>>;
    fn get_pull_server_lastcheckid(&self, server: &str) -> Option<String>;
    fn set_pull_server_lastcheckid(&self, server: &str, lastcheckid: &str) -> Result<(),Box<dyn std::error::Error>>;

    fn get_extra(&self, table_name: &str, column_name: &str, search: Option<String>, order: String, reverse: bool, hidebroken: bool) -> Result<Vec<ExtraInfo>, Box<dyn Error>>;
    fn get_1_n(&self, column: &str, search: Option<String>, order: String, reverse: bool, hidebroken: bool) -> Result<Vec<ExtraInfo>, Box<dyn Error>>;
    fn get_states(&self, country: Option<String>, search: Option<String>, order: String, reverse: bool, hidebroken: bool) -> Result<Vec<State>, Box<dyn Error>>;
    fn get_checks(&self, stationuuid: Option<String>, checkuuid: Option<String>, seconds: u32, include_history: bool) -> Result<Vec<StationCheckItem>, Box<dyn Error>>;

    fn insert_checks(&self, list: &Vec<StationCheckItemNew>) -> Result<(), Box<dyn Error>>;
    fn update_station_with_check_data(&self, list: &Vec<StationCheckItemNew>) -> Result<(), Box<dyn Error>>;

    fn delete_never_working(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
    fn delete_were_working(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
    fn delete_old_checks(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
    fn delete_old_clicks(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
}