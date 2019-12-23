use crate::db::models::StationCheckItem;
use crate::db::models::StationItem;
use crate::db::models::StationCheckItemNew;
use std::error::Error;
use crate::db::DbConnection;
use mysql;
use mysql::Row;
use mysql::QueryResult;

#[derive(Clone)]
pub struct MysqlConnection {
    pool: mysql::Pool,
}

impl From<Row> for StationCheckItem {
    fn from(mut row: Row) -> Self {
        StationCheckItem {
            check_id:       row.take("CheckID").unwrap(),
            station_uuid:   row.take("StationUuid").unwrap_or("".to_string()),
            check_uuid:     row.take("CheckUuid").unwrap_or("".to_string()),
            source:         row.take("Source").unwrap_or("".to_string()),
            codec:          row.take_opt("Codec").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            bitrate:        row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
            hls:            row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0) == 1,
            check_ok:       row.take_opt("CheckOK").unwrap_or(Ok(0)).unwrap_or(0) == 1,
            check_time:     row.take_opt("CheckTimeFormated").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url:            row.take_opt("UrlCache").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
        }
    }
}

impl From<Row> for StationItem {
    fn from(mut row: Row) -> Self {
        StationItem {
            id:              row.take("StationID").unwrap(),
            stationuuid:     row.take("StationUuid").unwrap_or("".to_string()),
            name:            row.take_opt("Name").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url:             row.take_opt("Url").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            url_resolved:    row.take_opt("UrlCache").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            codec:           row.take_opt("Codec").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            bitrate:         row.take_opt("Bitrate").unwrap_or(Ok(0)).unwrap_or(0),
            hls:             row.take_opt("Hls").unwrap_or(Ok(0)).unwrap_or(0)==1,
            lastcheckok:     row.take_opt("LastCheckOK").unwrap_or(Ok(0)).unwrap_or(0)==1,
            favicon:         row.take_opt("Favicon").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
            homepage:        row.take_opt("Homepage").unwrap_or(Ok("".to_string())).unwrap_or("".to_string()),
        }
    }
}

impl MysqlConnection {
    const COLUMNS: &'static str =
        "StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,UrlCache,
    Tags,Country,CountryCode,Subcountry,Language,Votes,
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

    pub fn new(connection_str: &str) -> Result<Self, Box<dyn Error>> {
        let pool = mysql::Pool::new(connection_str)?;
        Ok(
            MysqlConnection{
                pool,
            }
        )
    }

    fn get_list_from_query_result<'a, A>(&self, results: QueryResult<'static>,) -> Result<Vec<A>, Box<dyn Error>> where A: From<Row> {
        let mut list: Vec<A> = vec![];
        for result in results {
            let row = result?;
            list.push(row.into());
        }
        Ok(list)
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
        let delete_were_working_query = "DELETE FROM Station WHERE LastCheckOK=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < NOW() - INTERVAL :hours HOUR";
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
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOK=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < NOW() - INTERVAL :hours HOUR", params!(hours))
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

    /// Select all checks that are currently in the database of a station with the given uuid
    /// and calculate an overall status by majority vote. Ties are broken with the own vote
    /// of the most current check
    fn update_station_with_check_data(&mut self, list: Vec<&StationCheckItemNew>) -> Result<(), Box<dyn std::error::Error>> {
        let query_update_ok = "UPDATE Station SET LastCheckTime=NOW(),LastCheckOkTime=NOW(),LastCheckOK=:checkok,Codec=:codec,Bitrate=:bitrate,Hls=:hls,UrlCache=:urlcache WHERE StationUuid=:stationuuid";
        let mut stmt_update_ok = self.pool.prepare(query_update_ok)?;
        
        let query_update_not_ok = "UPDATE Station SET LastCheckTime=NOW(),LastCheckOK=:checkok,Codec=:codec,Bitrate=:bitrate,Hls=:hls,UrlCache=:urlcache WHERE StationUuid=:stationuuid";
        let mut stmt_update_not_ok = self.pool.prepare(query_update_not_ok)?;

        for item in list {
            // select all checks of the station
            let checks = self.get_checks(Some(item.station_uuid.clone()), None, 0)?;

            // calculate vote
            let all = checks.len();
            let mut ok: usize = 0;
            {
                for check in checks {
                    if check.check_ok {
                        ok += 1;
                    }
                }
            }
            let result;
            if ok == (all / 2) {
                // on ties -> the last check counts
                result = item.check_ok;
            }
            else if ok > (all / 2) {
                // majority positive
                result = true;
            }
            else
            {
                // majority negative
                result = false;
            }

            // update station with result
            trace!("Update station {} with {}/{} checks -> {}", item.station_uuid, ok, all, result);

            // insert into StationCheck
            let params = params!{
                "checkok" => result,
                "codec" => &item.codec,
                "bitrate" => item.bitrate,
                "hls" => item.hls,
                "urlcache" => &item.url,
                "stationuuid" => &item.station_uuid,
            };

            if result {
                stmt_update_ok.execute(params)?;
            } else {
                stmt_update_not_ok.execute(params)?;
            }
        }
        Ok(())
    }

    fn get_stations_to_check(&mut self, hours: u32, itemcount: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query = format!("SELECT {columns} FROM Station WHERE LastCheckTime IS NULL OR LastCheckTime < NOW() - INTERVAL {interval} HOUR ORDER BY RAND() LIMIT {limit}", columns = MysqlConnection::COLUMNS, interval = hours, limit = itemcount);
        let results = self.pool.prep_exec(query, ())?;
        self.get_list_from_query_result(results)
    }

    fn get_checks(&self, stationuuid: Option<String>, checkuuid: Option<String>, seconds: u32) -> Result<Vec<StationCheckItem>, Box<dyn Error>> {
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

                let query = format!("SELECT {columns} FROM StationCheckHistory WHERE StationUuid=? {where_checkuuid} {where_seconds} ORDER BY CheckTime", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str);
                self.pool.prep_exec(query, (uuid,))
            }
            None => {
                let where_checkuuid_str = if checkuuid.is_some() {
                    " AND CheckTime>=(SELECT CheckTime FROM StationCheck WHERE ChangeUuid=:checkuuid) AND ChangeUuid<>:checkuuid"
                } else {
                    ""
                };

                let query = format!("SELECT {columns} FROM StationCheck WHERE 1=1 {where_checkuuid} {where_seconds} ORDER BY CheckTime", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str);
                self.pool.prep_exec(query, ())
            }
        };

        self.get_list_from_query_result(results?)
    }
}