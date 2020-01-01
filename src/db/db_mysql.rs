use crate::db::models::State;
use crate::db::models::ExtraInfo;
use crate::db::models::StationItem;
use crate::db::models::StationCheckItem;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationChangeItemNew;
use std::error::Error;
use crate::db::DbConnection;
use mysql;
use mysql::Row;
use mysql::QueryResult;
use mysql::Value;

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
    Codec,Bitrate,Hls,LastCheckOK,
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

    fn backup_stations_by_uuid(transaction: &mut mysql::Transaction<'_>, stationuuids: &Vec<String>) -> Result<(),Box<dyn std::error::Error>>{
        let mut insert_params: Vec<Value> = vec![];
        let mut insert_query = vec![];
        for stationuuid in stationuuids {
            insert_params.push(stationuuid.into());
            insert_query.push("?");
        }

        let query = format!("INSERT INTO StationHistory(StationID,Name,Url,Homepage,Favicon,Country,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid)
                                                 SELECT StationID,Name,Url,Homepage,Favicon,Country,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid FROM Station WHERE StationUuid IN ({})", insert_query.join(","));
        let mut stmt = transaction.prepare(query)?;
        stmt.execute(insert_params)?;
        Ok(())
    }

    fn stationchange_exists(transaction: &mut mysql::Transaction<'_>, changeuuids: &Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut select_query = vec![];
        let mut select_params: Vec<Value> = vec![];
        for changeuuid in changeuuids {
            select_query.push("?");
            select_params.push(changeuuid.into());
        }
        let mut stmt = transaction.prepare(format!("SELECT ChangeUuid FROM StationHistory WHERE ChangeUuid IN ({})", select_query.join(",")))?;
        let result = stmt.execute(select_params)?;

        let mut list_result = vec![];
        for row in result {
            let (changeuuid,) = mysql::from_row_opt(row?)?;
            list_result.push(changeuuid);
        }
        Ok(list_result)
    }

    fn insert_station_by_change_internal(transaction: &mut mysql::Transaction<'_>, stationchanges: &Vec<StationChangeItemNew>) -> Result<Vec<String>,Box<dyn std::error::Error>> {
        // filter out changes that already exist in the database
        let changeuuids: Vec<String> = stationchanges.iter().map(|item|item.changeuuid.clone()).collect();
        let changeexists = MysqlConnection::stationchange_exists(transaction, &changeuuids)?;
        let mut list: Vec<&StationChangeItemNew> = vec![];
        for station in stationchanges {
            if !changeexists.contains(&station.changeuuid) {
                list.push(station);
            }
        }

        // insert changes
        let mut list_ids = vec![];
        if list.len() > 0 {
            let mut insert_query = vec![];
            let mut insert_params: Vec<Value> = vec![];
            for change in list {
                insert_query.push("(?,?,?,?,?,?,?,?,?,?,?,'')");
                insert_params.push(change.name.clone().into());
                insert_params.push(change.url.clone().into());
                insert_params.push(change.homepage.clone().into());
                insert_params.push(change.favicon.clone().into());
                insert_params.push(change.country.clone().into());
                insert_params.push(change.countrycode.clone().into());
                insert_params.push(change.state.clone().into());
                insert_params.push(fix_multi_field(&change.language).into());
                insert_params.push(fix_multi_field(&change.tags).into());
                insert_params.push(change.changeuuid.clone().into());
                insert_params.push(change.stationuuid.clone().into());
                list_ids.push(change.stationuuid.clone());
            }
            let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,CountryCode,Subcountry,Language,Tags,ChangeUuid,StationUuid, UrlCache) 
                                    VALUES{}", insert_query.join(","));
            let mut stmt = transaction.prepare(query)?;
            stmt.execute(insert_params)?;
        }
        Ok(list_ids)
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

    fn get_pull_server_lastid(&self, server: &str) -> Option<String> {
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

    fn set_pull_server_lastid(&self, server: &str, lastid: &str) -> Result<(),Box<dyn std::error::Error>> {
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

    fn get_pull_server_lastcheckid(&self, server: &str) -> Option<String> {
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

    fn set_pull_server_lastcheckid(&self, server: &str, lastcheckid: &str) -> Result<(),Box<dyn std::error::Error>> {
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

    fn insert_station_by_change(&self, list_station_changes: &Vec<StationChangeItemNew>) -> Result<Vec<String>,Box<dyn std::error::Error>> {
        let mut transaction = self.pool.start_transaction(false, None, None)?;

        let list_ids = MysqlConnection::insert_station_by_change_internal(&mut transaction, list_station_changes)?;
        MysqlConnection::backup_stations_by_uuid(&mut transaction, &list_ids)?;

        transaction.commit()?;
        Ok(list_ids)
    }

    fn insert_checks(&self, list: &Vec<StationCheckItemNew>) -> Result<(), Box<dyn std::error::Error>> {
        let mut transaction = self.pool.start_transaction(false, None, None)?;
        
        let mut delete_station_check_params: Vec<Value> = vec![];
        let mut delete_station_check_query = vec![];
        let mut insert_station_check_params: Vec<Value> = vec![];
        let mut insert_station_check_query = vec![];
        for item in list {
            delete_station_check_params.push(item.station_uuid.clone().into());
            delete_station_check_params.push(item.source.clone().into());
            delete_station_check_query.push("(StationUuid=? AND Source=?)");
  
            insert_station_check_params.push(item.station_uuid.clone().into());
            insert_station_check_params.push(item.source.clone().into());
            insert_station_check_params.push(item.codec.clone().into());
            insert_station_check_params.push(item.bitrate.into());
            insert_station_check_params.push(item.hls.into());
            insert_station_check_params.push(item.check_ok.into());
            insert_station_check_params.push(item.url.clone().into());
            insert_station_check_query.push("(?,UUID(),?,?,?,?,?,NOW(),?)");
        }

        {
            let query_delete_old_station_checks = format!("DELETE FROM StationCheck WHERE {}", delete_station_check_query.join(" OR "));
            let mut stmt_delete_old_station_checks = transaction.prepare(query_delete_old_station_checks)?;
            stmt_delete_old_station_checks.execute(delete_station_check_params)?;
        }

        let insert_station_check_params_str = insert_station_check_query.join(",");

        {
            let query_insert_station_check = format!("INSERT INTO StationCheck(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) VALUES{}", insert_station_check_params_str);
            let mut stmt_insert_station_check = transaction.prepare(query_insert_station_check)?;
            stmt_insert_station_check.execute(&insert_station_check_params)?;
        }

        {
            let query_insert_station_check_history = format!("INSERT INTO StationCheckHistory(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) VALUES{}", insert_station_check_params_str);
            let mut stmt_insert_station_check_history = transaction.prepare(query_insert_station_check_history)?;
            stmt_insert_station_check_history.execute(insert_station_check_params)?;
        }

        transaction.commit()?;

        Ok(())
    }

    /// Select all checks that are currently in the database of a station with the given uuid
    /// and calculate an overall status by majority vote. Ties are broken with the own vote
    /// of the most current check
    fn update_station_with_check_data(&self, list: &Vec<StationCheckItemNew>) -> Result<(), Box<dyn std::error::Error>> {
        let mut transaction = self.pool.start_transaction(false, None, None)?;

        let mut list_station_uuid = vec![];
        let mut list_station_uuid_query = vec![];

        {
            let query_update_ok = "UPDATE Station SET LastCheckTime=NOW(),LastCheckOkTime=NOW(),Codec=:codec,Bitrate=:bitrate,Hls=:hls,UrlCache=:urlcache WHERE StationUuid=:stationuuid";
            let mut stmt_update_ok = transaction.prepare(query_update_ok)?;
            
            for item in list {
                let params = params!{
                    "codec" => &item.codec,
                    "bitrate" => item.bitrate,
                    "hls" => item.hls,
                    "urlcache" => &item.url,
                    "stationuuid" => &item.station_uuid,
                };
                if item.check_ok {
                    stmt_update_ok.execute(params)?;
                }

                list_station_uuid.push(&item.station_uuid);
                list_station_uuid_query.push("?");
            }
        }

        {
            let query_in = list_station_uuid_query.join(",");
            let query_update_check_ok = format!("UPDATE Station st SET LastCheckTime=NOW(),LastCheckOk=(SELECT round(avg(CheckOk)) AS result FROM StationCheck sc WHERE sc.StationUuid=st.StationUuid GROUP BY StationUuid) WHERE StationUuid IN ({uuids});", uuids = query_in);
            let mut stmt_update_check_ok = transaction.prepare(query_update_check_ok)?;

            stmt_update_check_ok.execute(list_station_uuid)?;
        }

        transaction.commit()?;

        Ok(())
    }

    fn get_stations_to_check(&mut self, hours: u32, itemcount: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query = format!("SELECT {columns} FROM Station WHERE LastCheckTime IS NULL OR LastCheckTime < NOW() - INTERVAL {interval} HOUR ORDER BY RAND() LIMIT {limit}", columns = MysqlConnection::COLUMNS, interval = hours, limit = itemcount);
        let results = self.pool.prep_exec(query, ())?;
        self.get_list_from_query_result(results)
    }

    fn get_checks(&self, stationuuid: Option<String>, checkuuid: Option<String>, seconds: u32, include_history: bool) -> Result<Vec<StationCheckItem>, Box<dyn Error>> {
        let table_name = if include_history { "StationCheckHistory" } else { "StationCheck" };
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
                    " AND CheckTime>=(SELECT CheckTime FROM {table_name} WHERE ChangeUuid=:checkuuid) AND ChangeUuid<>:checkuuid"
                } else {
                    ""
                };

                let query = format!("SELECT {columns} FROM {table_name} WHERE StationUuid=? {where_checkuuid} {where_seconds} ORDER BY CheckTime", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str, table_name = table_name);
                self.pool.prep_exec(query, (uuid,))
            }
            None => {
                let where_checkuuid_str = if checkuuid.is_some() {
                    " AND CheckTime>=(SELECT CheckTime FROM {table_name} WHERE ChangeUuid=:checkuuid) AND ChangeUuid<>:checkuuid"
                } else {
                    ""
                };

                let query = format!("SELECT {columns} FROM {table_name} WHERE 1=1 {where_checkuuid} {where_seconds} ORDER BY CheckTime", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str, table_name = table_name);
                self.pool.prep_exec(query, ())
            }
        };

        self.get_list_from_query_result(results?)
    }

    fn get_extra(
        &self,
        table_name: &str,
        column_name: &str,
        search: Option<String>,
        order: String,
        reverse: bool,
        hidebroken: bool,
    ) -> Result<Vec<ExtraInfo>, Box<dyn Error>> {
        let mut params: Vec<Value> = Vec::with_capacity(1);
        let mut items = vec![];
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            "StationCountWorking as stationcount"
        } else {
            "StationCount as stationcount"
        };
        let search_string = match search {
            Some(c) => {
                params.push((format!("%{}%", c)).into());
                format!(" AND {} LIKE ?", column_name)
            }
            None => "".to_string(),
        };
        let mut stmt = self.pool.prepare(format!("SELECT {column_name} AS name, {hidebroken} FROM {table_name} WHERE {column_name} <> '' {search} HAVING stationcount > 0 ORDER BY {order} {reverse}",search = search_string, order = order, reverse = reverse_string, hidebroken = hidebroken_string, table_name = table_name, column_name = column_name))?;
        let result = stmt.execute(params)?;
        for row in result {
            let mut mut_row = row?;
            items.push(ExtraInfo::new(
                mut_row.take(0).unwrap_or("".into()),
                mut_row.take(1).unwrap_or(0),
            ));
        }
        Ok(items)
    }

    fn get_1_n(
        &self,
        column: &str,
        search: Option<String>,
        order: String,
        reverse: bool,
        hidebroken: bool,
    ) -> Result<Vec<ExtraInfo>, Box<dyn Error>> {
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
        }?;

        let mut stations = vec!();
        for row in result {
            let row = row?;
            let (name, stationcount) = mysql::from_row_opt(row)?;
            stations.push(ExtraInfo::new(name, stationcount));
        }
        Ok(stations)
    }

    fn get_states(
        &self,
        country: Option<String>,
        search: Option<String>,
        order: String,
        reverse: bool,
        hidebroken: bool,
    ) -> Result<Vec<State>, Box<dyn Error>> {
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

        let mut my_stmt = self.pool.prepare(format!(r"SELECT Subcountry AS name,Country,COUNT(*) AS stationcount FROM Station WHERE Subcountry <> '' {country} {search} {hidebroken} GROUP BY Subcountry, Country ORDER BY {order} {reverse}",hidebroken = hidebroken_string, order = order, country = country_string, reverse = reverse_string, search = search_string))?;
        let result = my_stmt.execute(params)?;
        let mut states: Vec<State> = vec![];

        for row in result {
            let row_unwrapped = row?;
            let (name, country, stationcount) = mysql::from_row_opt(row_unwrapped)?;
            states.push(State::new(name, country, stationcount));
        }
        Ok(states)
    }
}

fn fix_multi_field(value: &str) -> String {
    let values: Vec<String> = value.split(",").map(|v| v.trim().to_lowercase().to_string()).collect();
    values.join(",")
}