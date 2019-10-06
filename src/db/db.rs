use check::models::StationItem;
use check::models::StationCheckItemNew;
use std::error::Error;

pub trait DbConnection {
    fn delete_never_working(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
    fn delete_were_working(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
    fn delete_old_checks(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
    fn delete_old_clicks(&mut self, hours: u32) -> Result<(), Box<dyn Error>>;
    fn get_station_count_broken(&self) -> Result<u64, Box<dyn Error>>;
    fn get_station_count_working(&self) -> Result<u64, Box<dyn Error>>;
    fn get_station_count_todo(&self, hours: u32) -> Result<u64, Box<dyn Error>>;
    fn get_checks_todo_count(&self, hours: u32, source: &str) -> Result<u64, Box<dyn Error>>;
    fn get_deletable_never_working(&self, hours: u32) -> Result<u64, Box<dyn Error>>;
    fn get_deletable_were_working(&self, hours: u32) -> Result<u64, Box<dyn Error>>;

    fn insert_check(&mut self, item: &StationCheckItemNew) -> Result<(), Box<dyn std::error::Error>>;
    fn update_station(&mut self, item: &StationCheckItemNew) -> Result<(), Box<dyn std::error::Error>>;
    fn get_stations_to_check(&mut self, hours: u32, itemcount: u32) -> Vec<StationItem>;
}