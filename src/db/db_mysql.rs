use crate::check::models::StationItem;
use crate::check::models::StationCheckItemNew;
use std::error::Error;
use crate::db::DbConnection;
use mysql;

pub struct MysqlConnection {
    pool: mysql::Pool,
}

impl MysqlConnection {
    pub fn new(connection_str: &str) -> Result<Self, Box<dyn Error>> {
        let pool = mysql::Pool::new(connection_str)?;
        Ok(
            MysqlConnection{
                pool,
            }
        )
    }

    fn get_stations_query(&mut self, query: String) -> Vec<StationItem> {
        let mut stations: Vec<StationItem> = vec![];
        let results = self.pool.prep_exec(query, ());
        for result in results {
            for row_ in result {
                let mut row = row_.unwrap();
                let hls: i32 = row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0);
                let ok: i32 = row.take_opt("LastCheckOk").unwrap_or(Ok(0)).unwrap_or(0);
                let s = StationItem {
                    id:              row.take("StationID").unwrap(),
                    uuid:            row.take("StationUuid").unwrap_or("".to_string()),
                    name:            row.take_opt("Name").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    url:             row.take_opt("Url").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    urlcache:        row.take_opt("UrlCache").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    codec:           row.take_opt("Codec").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    bitrate:         row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
                    hls:             hls != 0,
                    check_ok:        ok != 0,
                    favicon:         row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                    homepage:        row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
                };
                stations.push(s);
            }
        }

        stations
    }

    pub fn get_single_column_number(&self, query: &str) -> Result<u64,Box<dyn std::error::Error>> {
        let results = self.pool.prep_exec(query, ())?;
        for result in results {
            let mut row = result?;
            let items: u64 = row.take_opt(0).unwrap_or(Ok(0))?;
            return Ok(items);
        }
        return Ok(0);
    }

    pub fn get_single_column_number_params(&self, query: &str, p: Vec<(String, mysql::Value)>) -> Result<u64,Box<dyn std::error::Error>> {
        let results = self.pool.prep_exec(query, p)?;
        for result in results {
            let mut row = result?;
            let items: u64 = row.take_opt(0).unwrap_or(Ok(0))?;
            return Ok(items);
        }
        return Ok(0);
    }
}

impl DbConnection for MysqlConnection {
    fn delete_old_checks(&mut self, hours: u32) -> Result<(), Box<dyn Error>> {
        let p = params!(hours);
        let delete_old_checks_history_query = "DELETE FROM StationCheckHistory WHERE CheckTime < NOW() - INTERVAL :hours HOUR";
        let mut delete_old_checks_history_stmt = self.pool.prepare(delete_old_checks_history_query)?;
        delete_old_checks_history_stmt.execute(&p)?;

        let delete_old_checks_current_query = "DELETE FROM StationCheck WHERE CheckTime < NOW() - INTERVAL :hours HOUR";
        let mut delete_old_checks_current_stmt = self.pool.prepare(delete_old_checks_current_query)?;
        delete_old_checks_current_stmt.execute(&p)?;
        Ok(())
    }

    fn delete_old_clicks(&mut self, hours: u32) -> Result<(), Box<dyn Error>> {
        let delete_old_clicks_query = "DELETE FROM StationClick WHERE ClickTimestamp < NOW() - INTERVAL :hours HOUR";
        let mut delete_old_clicks_stmt = self.pool.prepare(delete_old_clicks_query)?;
        delete_old_clicks_stmt.execute(params!(hours))?;
        Ok(())
    }

    fn delete_never_working(&mut self, hours: u32) -> Result<(), Box<dyn Error>> {
        let delete_never_working_query = "DELETE FROM Station WHERE LastCheckOkTime IS NULL AND Creation < NOW() - INTERVAL :hours HOUR";
        let mut delete_never_working_stmt = self.pool.prepare(delete_never_working_query)?;
        delete_never_working_stmt.execute(params!(hours))?;
        Ok(())
    }

    fn delete_were_working(&mut self, hours: u32) -> Result<(), Box<dyn Error>> {
        let delete_were_working_query = "DELETE FROM Station WHERE LastCheckOk=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < NOW() - INTERVAL :hours HOUR";
        let mut delete_were_working_stmt = self.pool.prepare(delete_were_working_query)?;
        delete_were_working_stmt.execute(params!(hours))?;
        Ok(())
    }

    fn get_station_count_broken(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number("SELECT COUNT(*) AS Items FROM radio.Station WHERE LastCheckOK=0 OR LastCheckOK IS NULL")
    }

    fn get_station_count_working(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number("SELECT COUNT(*) AS Items FROM radio.Station WHERE LastCheckOK=1")
    }

    fn get_tag_count(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number(r#"SELECT COUNT(*) AS StationCount FROM TagCache"#)
    }

    fn get_country_count(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number(r#"SELECT COUNT(DISTINCT(Country)) AS StationCount FROM Station"#)
    }

    fn get_language_count(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number(r#"SELECT COUNT(*) AS StationCount FROM LanguageCache"#)
    }

    fn get_click_count_last_hour(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number(r#"SELECT COUNT(*) FROM StationClick WHERE TIMESTAMPDIFF(MINUTE,ClickTimestamp,now())<=60;"#)
    }

    fn get_click_count_last_day(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number(r#"SELECT COUNT(*) FROM StationClick WHERE TIMESTAMPDIFF(HOUR,ClickTimestamp,now())<=24;"#)
    }

    fn get_station_count_todo(&self, hours: u32) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckTime IS NULL OR LastCheckTime < NOW() - INTERVAL :hours HOUR", params!(hours))
    }

    fn get_checks_todo_count(&self, hours: u32, source: &str) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM StationCheckHistory WHERE Source=:source AND CheckTime > NOW() - INTERVAL :hours HOUR",params!(hours, source))
    }

    fn get_deletable_never_working(&self, hours: u32) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOkTime IS NULL AND Creation < NOW() - INTERVAL :hours HOUR", params!(hours))
    }

    fn get_deletable_were_working(&self, hours: u32) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOk=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < NOW() - INTERVAL :hours HOUR", params!(hours))
    }

    fn insert_checks(&mut self, list: Vec<&StationCheckItemNew>) -> Result<(), Box<dyn std::error::Error>> {
        let query_delete_old_station_checks = "DELETE FROM StationCheck WHERE StationUuid=:stationuuid AND Source=:source";
        let query_insert_station_check = "INSERT INTO StationCheck(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) VALUES(?,UUID(),?,?,?,?,?,NOW(),?)";
        let query_insert_station_check_history = "INSERT INTO StationCheckHistory(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) VALUES(?,UUID(),?,?,?,?,?,NOW(),?)";

        let mut stmt_delete_old_station_checks = self.pool.prepare(query_delete_old_station_checks)?;
        let mut stmt_insert_station_check = self.pool.prepare(query_insert_station_check)?;
        let mut stmt_insert_station_check_history = self.pool.prepare(query_insert_station_check_history)?;

        for item in list {
            stmt_delete_old_station_checks.execute(params!(
                "stationuuid" => &item.station_uuid,
                "source" => &item.source
            ))?;
            stmt_insert_station_check.execute((&item.station_uuid,&item.source,&item.codec,&item.bitrate,&item.hls,&item.check_ok,&item.url))?;
            stmt_insert_station_check_history.execute((&item.station_uuid,&item.source,&item.codec,&item.bitrate,&item.hls,&item.check_ok,&item.url))?;
        }
        Ok(())
    }

    fn update_stations(&mut self, list: Vec<&StationCheckItemNew>) -> Result<(), Box<dyn std::error::Error>> {
        let query_update_ok = "UPDATE Station SET LastCheckTime=NOW(),LastCheckOkTime=NOW(),LastCheckOk=?,Codec=?,Bitrate=?,UrlCache=? WHERE StationUuid=?";
        let mut stmt_update_ok = self.pool.prepare(query_update_ok)?;
        
        let query_update_not_ok = "UPDATE Station SET LastCheckTime=NOW(),LastCheckOk=?,Codec=?,Bitrate=?,UrlCache=? WHERE StationUuid=?";
        let mut stmt_update_not_ok = self.pool.prepare(query_update_not_ok)?;

        for item in list {
            if item.check_ok{
                stmt_update_ok.execute((&item.check_ok,&item.codec,&item.bitrate,&item.url,&item.station_uuid))?;
            } else {
                stmt_update_not_ok.execute((&item.check_ok,&item.codec,&item.bitrate,&item.url,&item.station_uuid))?;
            }
        }
        Ok(())
    }

    fn get_stations_to_check(&mut self, hours: u32, itemcount: u32) -> Vec<StationItem> {
        let query = format!("SELECT StationID,StationUuid,Name,Codec,Bitrate,Hls,LastCheckOk,UrlCache,Url,Favicon,Homepage FROM Station WHERE LastCheckTime IS NULL OR LastCheckTime < NOW() - INTERVAL {} HOUR ORDER BY RAND() LIMIT {}", hours, itemcount);
        self.get_stations_query(query)
    }
}
