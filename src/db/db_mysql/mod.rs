mod migrations;
mod simple_migrate;
mod conversions;

use std::collections::HashSet;
use crate::db::db_error::DbError;

use std;
use std::collections::HashMap;

use crate::uuid::Uuid;
use crate::db::models::State;
use crate::db::models::ExtraInfo;
use crate::db::models::StationItem;
use crate::db::models::StationCheckItem;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationChangeItemNew;
use crate::db::models::StationClickItem;
use crate::db::models::StationClickItemNew;
use crate::db::models::StationHistoryItem;
use crate::api::data::Station;
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
    Date_Format(LastLocalCheckTime,'%Y-%m-%d %H:%i:%s') AS LastLocalCheckTimeFormated,
    ClickTimestamp,
    Date_Format(ClickTimestamp,'%Y-%m-%d %H:%i:%s') AS ClickTimestampFormated,
    clickcount,ClickTrend";

    const COLUMNS_CHECK: &'static str =
        "CheckID, StationUuid, CheckUuid, Source, Codec, Bitrate, Hls, CheckOK,
    CheckTime,
    Date_Format(CheckTime,'%Y-%m-%d %H:%i:%s') AS CheckTimeFormated,
    UrlCache,
    MetainfoOverridesDatabase,Public,Name,
    Description,Tags,CountryCode,
    Homepage,Favicon,Loadbalancer";

    const COLUMNS_CLICK: &'static str =
        "ClickID, StationUuid, ClickUuid, IP,
    Date_Format(ClickTimestamp,'%Y-%m-%d %H:%i:%s') AS ClickTimestampFormated";

    pub fn new(connection_str: &str) -> Result<Self, Box<dyn Error>> {
        let pool = mysql::Pool::new(connection_str)?;
        Ok(
            MysqlConnection{
                pool,
            }
        )
    }

    pub fn do_migrations(&self, ignore_migration_errors: bool, allow_database_downgrade: bool) -> Result<(), Box<dyn Error>> {
        let migrations = migrations::load_migrations(&self.pool)?;
        migrations.do_migrations(ignore_migration_errors, allow_database_downgrade)?;
        Ok(())
    }

    fn get_list_from_query_result<'a, A>(&self, results: QueryResult<'static>,) -> Result<Vec<A>, Box<dyn Error>> where A: From<Row> {
        let mut list: Vec<A> = vec![];
        for result in results {
            let row = result?;
            list.push(row.into());
        }
        Ok(list)
    }

    fn get_stations_query(&self, query: String) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let results = self.pool.prep_exec(query, ())?;
        self.get_list_from_query_result(results)
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
        if stationuuids.len() > 0{
            let mut insert_params: Vec<Value> = vec![];
            let mut insert_query = vec![];
            for stationuuid in stationuuids {
                insert_params.push(stationuuid.into());
                insert_query.push("?");
            }
            let query = format!("INSERT INTO StationHistory(Name,Url,Homepage,Favicon,Country,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid)
                                                     SELECT Name,Url,Homepage,Favicon,Country,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid FROM Station WHERE StationUuid IN ({})", insert_query.join(","));
            let mut stmt = transaction.prepare(query)?;
            stmt.execute(insert_params)?;
        }
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

        trace!("Ignored changes for insert: {}", changeexists.len());

        // insert changes
        let mut list_ids = vec![];
        if list.len() > 0 {
            let mut insert_query = vec![];
            let mut insert_params: Vec<Value> = vec![];
            for change in list {
                insert_query.push("(?,?,?,?,?,?,?,?,?,?,?,'',UTC_TIMESTAMP())");
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
            let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,CountryCode,Subcountry,Language,Tags,ChangeUuid,StationUuid, UrlCache, Creation) 
                                    VALUES{}", insert_query.join(","));
            let mut stmt = transaction.prepare(query)?;
            stmt.execute(insert_params)?;
        }
        Ok(list_ids)
    }

    /*
    pub fn prefill_station_click_from_history(&self) {
        INSERT INTO StationCheck (CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer)
                           SELECT CheckUuid,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache,MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer FROM StationCheckHistory
                                  WHERE CheckID IN (select max(CheckID) FROM StationCheckHistory Group By StationUuid,Source);
    }
    */
}

impl DbConnection for MysqlConnection {
    fn delete_old_checks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let p = params!(seconds);
        let delete_old_checks_history_query = "DELETE FROM StationCheckHistory WHERE CheckTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut delete_old_checks_history_stmt = self.pool.prepare(delete_old_checks_history_query)?;
        delete_old_checks_history_stmt.execute(&p)?;

        Ok(())
    }

    fn delete_old_clicks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let delete_old_clicks_query = "DELETE FROM StationClick WHERE ClickTimestamp < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut delete_old_clicks_stmt = self.pool.prepare(delete_old_clicks_query)?;
        delete_old_clicks_stmt.execute(params!(seconds))?;
        Ok(())
    }

    fn delete_never_working(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let delete_never_working_query = "DELETE FROM Station WHERE LastCheckOkTime IS NULL AND Creation < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut delete_never_working_stmt = self.pool.prepare(delete_never_working_query)?;
        delete_never_working_stmt.execute(params!(seconds))?;
        Ok(())
    }

    fn delete_were_working(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let delete_were_working_query = "DELETE FROM Station WHERE LastCheckOK=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut delete_were_working_stmt = self.pool.prepare(delete_were_working_query)?;
        delete_were_working_stmt.execute(params!(seconds))?;
        Ok(())
    }

    fn remove_unused_ip_infos_from_stationclicks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let query = "UPDATE StationClick SET IP=NULL WHERE InsertTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut stmt = self.pool.prepare(query)?;
        stmt.execute(params!(seconds))?;
        Ok(())
    }

    fn remove_illegal_icon_links(&mut self) -> Result<(), Box<dyn Error>> {
        let query = r#"UPDATE Station SET Favicon="" WHERE LOWER(Favicon) NOT LIKE 'http://%' AND LOWER(Favicon) NOT LIKE'https://%' AND Favicon<>"";"#;
        self.pool.prep_exec(query, ())?;
        Ok(())
    }

    fn update_stations_clickcount(&self) -> Result<(), Box<dyn Error>> {
        trace!("update_stations_clickcount() 1");
        let query = "UPDATE Station st SET 
        clickcount=IFNULL((SELECT COUNT(*) FROM StationClick sc WHERE st.StationUuid=sc.StationUuid),0),
        ClickTrend=
        (
            (select count(*) from StationClick sc1 where sc1.StationUuid=st.StationUuid AND ClickTimestamp>DATE_SUB(UTC_TIMESTAMP(),INTERVAL 1 DAY) AND ClickTimestamp<=DATE_SUB(UTC_TIMESTAMP(),INTERVAL 0 DAY)) - 
            (select count(*) from StationClick sc2 where sc2.StationUuid=st.StationUuid AND ClickTimestamp>DATE_SUB(UTC_TIMESTAMP(),INTERVAL 2 DAY) AND ClickTimestamp<=DATE_SUB(UTC_TIMESTAMP(),INTERVAL 1 DAY))
        ),
        ClickTimestamp=(SELECT Max(ClickTimestamp) FROM StationClick sc WHERE sc.StationUuid=st.StationUuid);";
        let mut stmt = self.pool.prepare(query)?;
        stmt.execute(())?;
        trace!("update_stations_clickcount() 2");
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
        self.get_single_column_number(r#"SELECT COUNT(*) FROM StationClick WHERE TIMESTAMPDIFF(MINUTE,ClickTimestamp,UTC_TIMESTAMP())<=60;"#)
    }

    fn get_click_count_last_day(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number(r#"SELECT COUNT(*) FROM StationClick WHERE TIMESTAMPDIFF(HOUR,ClickTimestamp,UTC_TIMESTAMP())<=24;"#)
    }

    /**
     * Get number of stations that do not have any checks in the last x hours
     */
    fn get_station_count_todo(&self, hours: u32) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastLocalCheckTime IS NULL OR LastLocalCheckTime < UTC_TIMESTAMP() - INTERVAL :hours HOUR", params!(hours))
    }

    fn get_stations_to_check(&mut self, hours: u32, itemcount: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query = format!("SELECT {columns} FROM Station WHERE LastLocalCheckTime IS NULL OR LastLocalCheckTime < UTC_TIMESTAMP() - INTERVAL {interval} HOUR ORDER BY RAND() LIMIT {limit}", columns = MysqlConnection::COLUMNS, interval = hours, limit = itemcount);
        let results = self.pool.prep_exec(query, ())?;
        self.get_list_from_query_result(results)
    }

    fn get_station_by_uuid(&self, id_str: &str) -> Result<Vec<StationItem>,Box<dyn Error>> {
        let query = format!(
            "SELECT {columns} from Station WHERE StationUuid=? ORDER BY Name",
            columns = MysqlConnection::COLUMNS
        );
        let results = self.pool.prep_exec(query, (id_str,))?;
        self.get_list_from_query_result(results)
    }

    fn get_deletable_never_working(&self, seconds: u64) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOkTime IS NULL AND Creation < UTC_TIMESTAMP() - INTERVAL :seconds SECOND", params!(seconds))
    }

    fn get_deletable_were_working(&self, seconds: u64) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOK=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND", params!(seconds))
    }

    fn get_stations_broken(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        self.get_stations_query(format!(
            "SELECT {columns} from Station WHERE LastCheckOK=FALSE ORDER BY rand() LIMIT {limit}",
            columns = MysqlConnection::COLUMNS,
            limit = limit
        ))
    }

    fn get_stations_improvable(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        self.get_stations_query(format!(r#"SELECT {columns} from Station WHERE LastCheckOK=TRUE AND (Tags="" OR Country="") ORDER BY RAND() LIMIT {limit}"#,columns = MysqlConnection::COLUMNS, limit = limit))
    }

    fn get_stations_topvote(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY Votes DESC LIMIT {limit}",
            columns = MysqlConnection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    fn get_stations_topclick(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY clickcount DESC LIMIT {limit}",
            columns = MysqlConnection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    fn get_stations_lastclick(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY ClickTimestamp DESC LIMIT {limit}",
            columns = MysqlConnection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    fn get_stations_lastchange(&self, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        query = format!(
            "SELECT {columns} from Station ORDER BY Creation DESC LIMIT {limit}",
            columns = MysqlConnection::COLUMNS,
            limit = limit
        );
        self.get_stations_query(query)
    }

    fn get_stations_by_column(
        &self,
        column_name: &str,
        search: String,
        exact: bool,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let order = filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let query: String = if exact {
            format!("SELECT {columns} from Station WHERE LOWER({column_name})=? {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = MysqlConnection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        } else {
            format!("SELECT {columns} from Station WHERE LOWER({column_name}) LIKE CONCAT('%',?,'%') {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = MysqlConnection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        };
        let results = self.pool.prep_exec(query, (search.to_lowercase(),))?;
        self.get_list_from_query_result(results)
    }

    fn get_stations_by_column_multiple(
        &self,
        column_name: &str,
        search: Option<String>,
        exact: bool,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let order = filter_order(order);
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
                columns = MysqlConnection::COLUMNS,
                order = order,
                reverse = reverse_string,
                hidebroken = hidebroken_string,
                offset = offset,
                limit = limit,
                column_name = column_name
            )
        } else {
            format!("SELECT {columns} from Station WHERE {column_name} LIKE CONCAT('%',?,'%') {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}", columns = MysqlConnection::COLUMNS, order = order, reverse = reverse_string, hidebroken = hidebroken_string, offset = offset, limit = limit, column_name = column_name)
        };
        let results = if exact {
            self.pool.prep_exec(query, (&search, &search, &search, &search))?
        } else {
            self.pool.prep_exec(query, (search,))?
        };
        self.get_list_from_query_result(results)
    }

    fn get_stations_by_all(
        &self,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let order = filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " WHERE LastCheckOK=TRUE"
        } else {
            ""
        };

        let query: String = format!("SELECT {columns} from Station {hidebroken} ORDER BY {order} {reverse} LIMIT {offset},{limit}",
            columns = MysqlConnection::COLUMNS, order = order, reverse = reverse_string,
            hidebroken = hidebroken_string, offset = offset, limit = limit);
        let results = self.pool.prep_exec(query, ())?;
        self.get_list_from_query_result(results)
    }

    fn get_stations_advanced(
        &self,
        name: Option<String>,
        name_exact: bool,
        country: Option<String>,
        country_exact: bool,
        countrycode: Option<String>,
        state: Option<String>,
        state_exact: bool,
        language: Option<String>,
        language_exact: bool,
        tag: Option<String>,
        tag_exact: bool,
        tag_list: Vec<String>,
        codec: Option<String>,
        bitrate_min: u32,
        bitrate_max: u32,
        order: &str,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let order = filter_order(order);
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let mut query = format!(
            "SELECT {columns} from Station WHERE",
            columns = MysqlConnection::COLUMNS
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
        if countrycode.is_some() {
            query.push_str(" AND UPPER(CountryCode)=UPPER(:countrycode)");
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
        if codec.is_some() {
            query.push_str(" AND Codec=:codec");
        }
        let mut params = params!{
            "name" => name.unwrap_or_default(),
            "country" => country.unwrap_or_default(),
            "countrycode" => countrycode.unwrap_or_default(),
            "state" => state.unwrap_or_default(),
            "language" => language.unwrap_or_default(),
            "tag" => tag.unwrap_or_default(),
            "codec" => codec.unwrap_or_default(),
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
        )?;
        self.get_list_from_query_result(results)
    }

    fn get_changes(&self, stationuuid: Option<String>, changeuuid: Option<String>) -> Result<Vec<StationHistoryItem>, Box<dyn Error>> {
        let changeuuid_str = if changeuuid.is_some() {
            " AND StationChangeID >= IFNULL((SELECT StationChangeID FROM StationHistory WHERE ChangeUuid=:changeuuid),0)
              AND StationChangeID <= (SELECT MAX(StationChangeID) FROM StationHistory WHERE Creation <= UTC_TIMESTAMP() - INTERVAL 60 SECOND)
              AND ChangeUuid<>:changeuuid"
        } else {
            ""
        };

        let stationuuid_str = if stationuuid.is_some() {
            " AND StationUuid=:stationuuid"
        }else{
            ""
        };
        
        let query: String = format!("SELECT StationChangeID,ChangeUuid,
                StationUuid,Name,
                Url,Homepage,
                Favicon,Tags,
                Country,Subcountry,
                CountryCode,
                Language,Votes,
                Date_Format(Creation,'%Y-%m-%d %H:%i:%s') AS CreationFormated
                from StationHistory WHERE 1=:mynumber {changeuuid_str} {stationuuid} ORDER BY StationChangeID ASC", changeuuid_str = changeuuid_str, stationuuid = stationuuid_str);
        let results = self.pool.prep_exec(query, params! {
            "mynumber" => 1,
            "stationuuid" => stationuuid.unwrap_or(String::from("")),
            "changeuuid" => changeuuid.unwrap_or(String::from(""))
        })?;
        self.get_list_from_query_result(results)
    }

    fn add_station_opt(&self, name: Option<String>, url: Option<String>, homepage: Option<String>, favicon: Option<String>,
        country: Option<String>, countrycode: Option<String>, state: Option<String>, language: Option<String>, tags: Option<String>) -> Result<String, Box<dyn Error>> {
        let mut transaction = self.pool.start_transaction(false, None, None)?;

        let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,CountryCode,Subcountry,Language,Tags,ChangeUuid,StationUuid, UrlCache,Creation) 
                        VALUES(:name, :url, :homepage, :favicon, :country, :countrycode, :state, :language, :tags, :changeuuid, :stationuuid, '', UTC_TIMESTAMP())");

        let name = name.ok_or(DbError::AddStationError(String::from("name is empty")))?;
        let url = url.ok_or(DbError::AddStationError(String::from("url is empty")))?;
        
        if name.len() > 400{
            return Err(Box::new(DbError::AddStationError(String::from("name is longer than 400 chars"))));
        }

        let stationuuid = Uuid::new_v4().to_hyphenated().to_string();
        let changeuuid = Uuid::new_v4().to_hyphenated().to_string();
        let params = params!{
            "name" => name,
            "url" => url,
            "homepage" => homepage.unwrap_or_default(),
            "favicon" => favicon.unwrap_or_default(),
            "country" => country.unwrap_or_default(),
            "countrycode" => countrycode.unwrap_or_default(),
            "state" => state.unwrap_or_default(),
            "language" => fix_multi_field(&language.unwrap_or_default()),
            "tags" => fix_multi_field(&tags.unwrap_or_default()),
            "changeuuid" => changeuuid,
            "stationuuid" => stationuuid.clone(),
        };

        transaction.prep_exec(query, params)?;
        MysqlConnection::backup_stations_by_uuid(&mut transaction, &(vec![stationuuid.clone()]))?;
        transaction.commit()?;

        Ok(stationuuid)
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


    fn get_pull_server_lastclickid(&self, server: &str) -> Option<String> {
        let query: String = format!("SELECT lastclickuuid FROM PullServers WHERE name=:name");
        let results = self.pool.prep_exec(query, params!{
            "name" => server
        });
        match results {
            Ok(results) => {
                for result in results {
                    if let Ok(mut result) = result {
                        let lastclickuuid = result.take_opt("lastclickuuid");
                        if let Some(lastclickuuid) = lastclickuuid {
                            if let Ok(lastclickuuid) = lastclickuuid {
                                return Some(lastclickuuid);
                            }
                        }
                    }
                };
                None
            },
            _ => None
        }
    }

    fn set_pull_server_lastclickid(&self, server: &str, lastclickuuid: &str) -> Result<(),Box<dyn std::error::Error>> {
        let params = params!{
            "name" => server,
            "lastclickuuid" => lastclickuuid,
        };
        let query_update: String = format!("UPDATE PullServers SET lastclickuuid=:lastclickuuid WHERE name=:name");
        let results_update = self.pool.prep_exec(query_update, &params)?;
        if results_update.affected_rows() == 0 {
            let query_insert: String = format!("INSERT INTO PullServers(name, lastclickuuid) VALUES(:name,:lastclickuuid)");
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

    fn insert_checks(&self, list: &Vec<StationCheckItemNew>) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
        trace!("insert_checks()");
        let mut transaction = self.pool.start_transaction(false, None, None)?;
        
        // search for checkuuids in history table, if already added (maybe from other source)
        let mut existing_checks: HashSet<String> = HashSet::new();
        {
            let search_params: Vec<Value> = list.iter().filter_map(|item| item.checkuuid.clone()).map(|item2| item2.into()).collect();
            let search_query: Vec<&str> = (0..search_params.len()).map(|_item| "?").collect();

            if search_query.len() > 0 {
                let query_delete_old_station_checks = format!("SELECT CheckUuid FROM StationCheckHistory WHERE CheckUuid IN ({})", search_query.join(","));
                let mut stmt_delete_old_station_checks = transaction.prepare(query_delete_old_station_checks)?;
                let result = stmt_delete_old_station_checks.execute(search_params)?;

                for row in result {
                    let (checkuuid, ) = mysql::from_row_opt(row?)?;
                    existing_checks.replace(checkuuid);
                }
            }
        }

        trace!("Ignored checks(already existing) for insert: {}", existing_checks.len());

        // search for stations by stationuuid
        let mut existing_stations: HashSet<String> = HashSet::new();
        {
            let search_params: Vec<Value> = list.iter().map(|item| item.station_uuid.clone().into()).collect();
            let search_query: Vec<&str> = (0..search_params.len()).map(|_item| "?").collect();

            if search_query.len() > 0 {
                let query_select_stations_by_uuid = format!("SELECT StationUuid FROM Station WHERE StationUuid IN ({})", search_query.join(","));
                let mut stmt_select_stations_by_uuid = transaction.prepare(query_select_stations_by_uuid)?;
                let result = stmt_select_stations_by_uuid.execute(search_params)?;

                for row in result {
                    let (stationuuid, ) = mysql::from_row_opt(row?)?;
                    existing_stations.replace(stationuuid);
                }
            }
        }

        trace!("Found stations {}", existing_stations.len());

        // create lists for insertion
        let mut delete_station_check_params: Vec<Value> = vec![];
        let mut delete_station_check_query = vec![];
        let mut insert_station_check_params: Vec<Value> = vec![];
        let mut insert_station_check_query = vec![];
        let mut ignored_checks_no_station = 0;
        for item in list {
            // ignore checks, where there is no station in the database
            if !existing_stations.contains(&item.station_uuid) {
                ignored_checks_no_station += 1;
                continue;
            }
            // check has checkuuid ?
            match &item.checkuuid {
                Some(checkuuid) => {
                    // ignore checks that are already in the database
                    if existing_checks.contains(checkuuid) {
                        continue;
                    }
                    // reuse checkuuid
                    match &item.timestamp {
                        Some(timestamp) => {
                            insert_station_check_query.push("(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP())");
                            insert_station_check_params.push(checkuuid.into());
                            insert_station_check_params.push(timestamp.into());
                        }
                        None => {
                            insert_station_check_query.push("(?,UTC_TIMESTAMP(),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP())");
                            insert_station_check_params.push(checkuuid.into());
                        }
                    }
                },
                None => {
                    // generate new checkuuid
                    match &item.timestamp {
                        Some(timestamp) => {
                            insert_station_check_query.push("(UUID(),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP())");
                            insert_station_check_params.push(timestamp.into());
                        }
                        None => {
                            insert_station_check_query.push("(UUID(),UTC_TIMESTAMP(),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP())");
                        }
                    }
                    
                }
            }
            
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

            insert_station_check_params.push(item.metainfo_overrides_database.clone().into());
            insert_station_check_params.push(item.public.clone().into());
            insert_station_check_params.push(item.name.clone().into());
            insert_station_check_params.push(item.description.clone().into());
            insert_station_check_params.push(item.tags.clone().into());
            insert_station_check_params.push(item.countrycode.clone().into());
            insert_station_check_params.push(item.homepage.clone().into());
            insert_station_check_params.push(item.favicon.clone().into());
            insert_station_check_params.push(item.loadbalancer.clone().into());
        }

        trace!("Ignored checks(no stations) for insert: {}", ignored_checks_no_station);

        // insert into history table
        if insert_station_check_query.len() > 0 {
            let insert_station_check_params_str = insert_station_check_query.join(",");
            let query_insert_station_check_history = format!("INSERT INTO StationCheckHistory(CheckUuid,CheckTime,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,UrlCache,
                MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,InsertTime) VALUES{}", insert_station_check_params_str);
            let mut stmt_insert_station_check_history = transaction.prepare(query_insert_station_check_history)?;
            stmt_insert_station_check_history.execute(&insert_station_check_params)?;
        }

        transaction.commit()?;

        Ok(existing_checks)
    }

    /// Select all checks that are currently in the database of a station with the given uuid
    /// and calculate an overall status by majority vote. Ties are broken with the own vote
    /// of the most current check
    fn update_station_with_check_data(&self, list: &Vec<StationCheckItemNew>, local: bool) -> Result<(), Box<dyn std::error::Error>> {
        trace!("update_station_with_check_data()");
        let mut transaction = self.pool.start_transaction(false, None, None)?;

        let mut list_station_uuid = vec![];
        let mut list_station_uuid_query = vec![];

        for item in list {
            list_station_uuid.push(&item.station_uuid);
            list_station_uuid_query.push("?");
        }
        let query_in = list_station_uuid_query.join(",");

        let mut majority_vote: HashMap<String,bool> = HashMap::new();
        if list.len() > 0 {
            // calculate majority vote for checks
            let mut stmt_update_ok = transaction.prepare(format!("SELECT StationUuid,ROUND(AVG(CheckOk)) AS result FROM StationCheck WHERE StationUuid IN ({}) GROUP BY StationUuid", uuids = query_in))?;
            let result = stmt_update_ok.execute(&list_station_uuid)?;

            for row in result {
                let (stationuuid, result): (String, u8,) = mysql::from_row_opt(row?)?;
                majority_vote.insert(stationuuid, result == 1);
            }
        }

        {
            for item in list {
                let vote = majority_vote.get(&item.station_uuid).unwrap_or(&true);

                let mut params = params!{
                    "codec" => &item.codec,
                    "bitrate" => item.bitrate,
                    "hls" => item.hls,
                    "stationuuid" => &item.station_uuid,
                    "vote" => vote,
                };

                if item.metainfo_overrides_database {
                    let mut query = vec![];
                    let public = item.public.unwrap_or(true);
                    if public {
                        if let Some(name) = &item.name {
                            params.push((String::from("name"),name.into(),));
                            query.push("Name=:name");
                        }
                        if let Some(homepage) = &item.homepage {
                            params.push((String::from("homepage"),homepage.into(),));
                            query.push("Homepage=:homepage");
                        }
                        if let Some(loadbalancer) = &item.loadbalancer {
                            params.push((String::from("urlcache"),loadbalancer.into(),));
                        }
                        if let Some(countrycode) = &item.countrycode {
                            params.push((String::from("countrycode"),countrycode.into(),));
                            query.push("CountryCode=:countrycode");
                        }
                        if let Some(tags) = &item.tags {
                            params.push((String::from("tags"),fix_multi_field(tags).into(),));
                            query.push("Tags=:tags");
                        }
                        if let Some(favicon) = &item.favicon {
                            params.push((String::from("favicon"),favicon.into(),));
                            query.push("Favicon=:favicon");
                        }
                        query.push("LastCheckOk=:vote");
                        if local {
                            query.push("LastLocalCheckTime=UTC_TIMESTAMP()");
                        }

                        if item.check_ok {
                            let query_update_ok = format!("UPDATE Station SET LastCheckOkTime=UTC_TIMESTAMP(),LastCheckTime=UTC_TIMESTAMP(),Codec=:codec,Bitrate=:bitrate,Hls=:hls,UrlCache=:urlcache,{} WHERE StationUuid=:stationuuid", query.join(","));
                            let mut stmt_update_ok = transaction.prepare(query_update_ok)?;
                            stmt_update_ok.execute(params)?;
                        }
                    }else{
                        let query_delete = "DELETE FROM Station WHERE StationUuid=:stationuuid";
                        let mut stmt_delete = transaction.prepare(query_delete)?;
                        stmt_delete.execute(params)?;
                    }
                }else{
                    if item.check_ok {
                        params.push((String::from("urlcache"), item.url.clone().into(),));

                        let query_update_ok = format!("UPDATE Station SET {lastlocalchecktime}LastCheckOkTime=UTC_TIMESTAMP(),LastCheckTime=UTC_TIMESTAMP(),Codec=:codec,Bitrate=:bitrate,Hls=:hls,UrlCache=:urlcache,LastCheckOk=:vote WHERE StationUuid=:stationuuid",
                            lastlocalchecktime = if local {"LastLocalCheckTime=UTC_TIMESTAMP(),"} else {""},
                        );
                        let mut stmt_update_ok = transaction.prepare(query_update_ok)?;
                        stmt_update_ok.execute(params)?;
                    }else{
                        let query_update_check_ok = format!("UPDATE Station st SET {lastlocalchecktime}LastCheckTime=UTC_TIMESTAMP(),LastCheckOk=:vote WHERE StationUuid=:stationuuid",
                            lastlocalchecktime = if local {"LastLocalCheckTime=UTC_TIMESTAMP(),"} else {""},
                        );
                        let mut stmt_update_check_ok = transaction.prepare(query_update_check_ok)?;
                        stmt_update_check_ok.execute(params)?;
                    }
                }
            }
        }
        transaction.commit()?;

        Ok(())
    }

    fn insert_clicks(&self, list: &Vec<StationClickItemNew>) -> Result<(), Box<dyn Error>> {
        let mut transaction = self.pool.start_transaction(false, None, None)?;

        let mut found_clickuuids: Vec<String> = vec![];
        {
            let mut search_click_params: Vec<Value> = vec![];
            let mut search_click_query = vec![];
            
            for item in list {
                search_click_params.push(item.clickuuid.clone().into());
                search_click_query.push("?");
            }
            {
                let query = format!("SELECT ClickUuid FROM StationClick WHERE ClickUuid IN ({})", search_click_query.join(","));
                let mut stmt_search_clicks = transaction.prepare(query)?;
                let result = stmt_search_clicks.execute(search_click_params)?;
                for row in result {
                    let (clickuuid,) = mysql::from_row_opt(row?)?;
                    found_clickuuids.push(clickuuid);
                }
            }
        }

        trace!("Ignored clicks(already existing) for insert: {}", found_clickuuids.len());

        let mut found_stationuuids: Vec<String> = vec![];
        {
            let mut search_station_params: Vec<Value> = vec![];
            let mut search_station_query = vec![];
            for item in list {
                search_station_params.push(item.stationuuid.clone().into());
                search_station_query.push("?");
            }
            {
                let query = format!("SELECT StationUuid FROM Station WHERE StationUuid IN ({})", search_station_query.join(","));
                let mut stmt_search_clicks = transaction.prepare(query)?;
                let result = stmt_search_clicks.execute(search_station_params)?;
                for row in result {
                    let (stationuuid,) = mysql::from_row_opt(row?)?;
                    found_stationuuids.push(stationuuid);
                }
            }
        }

        let mut insert_click_params: Vec<Value> = vec![];
        let mut insert_click_query = vec![];
        let mut ignored_clicks = 0;
        for item in list {
            if !found_stationuuids.contains(&item.stationuuid) {
                ignored_clicks += 1;
                continue;
            }
            if !found_clickuuids.contains(&item.clickuuid) {
                insert_click_params.push(item.clickuuid.clone().into());
                insert_click_params.push(item.stationuuid.clone().into());
                insert_click_params.push(item.clicktimestamp.clone().into());

                insert_click_query.push("(?,?,?,UTC_TIMESTAMP())");
            }
        }

        trace!("Ignored clicks(no stations) for insert: {}", ignored_clicks);

        if insert_click_query.len() > 0 {
            let query = format!("INSERT INTO StationClick(ClickUuid, StationUuid, ClickTimestamp, InsertTime) VALUES{}", insert_click_query.join(","));
            let mut stmt_insert = transaction.prepare(query)?;
            stmt_insert.execute(insert_click_params)?;
        }

        transaction.commit()?;

        Ok(())
    }

    fn get_checks(&self, stationuuid: Option<String>, checkuuid: Option<String>, seconds: u32, include_history: bool) -> Result<Vec<StationCheckItem>, Box<dyn Error>> {
        let table_name = if include_history { "StationCheckHistory" } else { "StationCheck" };
        let where_seconds = if seconds > 0 {
            format!(
                "AND TIMESTAMPDIFF(SECOND,CheckTime,UTC_TIMESTAMP())<{seconds}",
                seconds = seconds
            )
        } else {
            String::from("")
        };

        let mut query_params = params!{"one" => 1};
        let where_checkuuid_str = match checkuuid {
            Some(checkuuid) => {
                query_params.push((String::from("checkuuid"), checkuuid.into(),));
                format!(" AND CheckID >= IFNULL((SELECT CheckID FROM StationCheckHistory WHERE CheckUuid=:checkuuid),0)
                          AND CheckID <= (SELECT MAX(CheckID) FROM StationCheckHistory WHERE InsertTime <= UTC_TIMESTAMP() - INTERVAL 60 SECOND)
                          AND CheckUuid<>:checkuuid")
            },
            None => String::from("")
        };

        let query = match stationuuid {
            Some(stationuuid) => {
                query_params.push((String::from("stationuuid"), stationuuid.into(),));
                format!("SELECT {columns} FROM {table_name} WHERE StationUuid=:stationuuid {where_checkuuid} {where_seconds} ORDER BY CheckID", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str, table_name = table_name)
            }
            None => {
                format!("SELECT {columns} FROM {table_name} WHERE 1=:one {where_checkuuid} {where_seconds} ORDER BY CheckID", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str, table_name = table_name)
            }
        };

        trace!("get_checks() {}", query);
        let results = self.pool.prep_exec(query, query_params);

        self.get_list_from_query_result(results?)
    }

    fn get_clicks(&self, stationuuid: Option<String>, clickuuid: Option<String>, seconds: u32) -> Result<Vec<StationClickItem>, Box<dyn Error>> {
        let where_seconds = if seconds > 0 {
            format!(
                "AND TIMESTAMPDIFF(SECOND,ClickTimestamp,UTC_TIMESTAMP())<{seconds}",
                seconds = seconds
            )
        } else {
            String::from("")
        };

        let mut query_params = params!{"one" => 1};
        let where_clickuuid_str = match clickuuid {
            Some(clickuuid) => {
                query_params.push((String::from("clickuuid"), clickuuid.into(),));
                " AND ClickID >= IFNULL((SELECT ClickID FROM StationClick WHERE ClickUuid=:clickuuid),0)
                  AND ClickID <= (SELECT MAX(ClickID) FROM StationClick WHERE InsertTime <= UTC_TIMESTAMP() - INTERVAL 60 SECOND)
                  AND ClickUuid<>:clickuuid"
            },
            None => ""
        };
        let query = match stationuuid {
            Some(stationuuid) => {
                query_params.push((String::from("stationuuid"), stationuuid.into(),));
                format!("SELECT {columns} FROM StationClick WHERE StationUuid=:stationuuid {where_clickuuid} {where_seconds} ORDER BY ClickID LIMIT 10000", columns = MysqlConnection::COLUMNS_CLICK, where_seconds = where_seconds, where_clickuuid = where_clickuuid_str)
            }
            None => {
                format!("SELECT {columns} FROM StationClick WHERE 1=:one {where_clickuuid} {where_seconds} ORDER BY ClickID LIMIT 10000", columns = MysqlConnection::COLUMNS_CLICK, where_seconds = where_seconds, where_clickuuid = where_clickuuid_str)
            }
        };

        trace!("get_clicks() {}", query);
        let results = self.pool.prep_exec(query, query_params);

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
        let order = filter_order_1_n(&order)?;
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
        let order = filter_order_1_n(&order)?;
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

    /// Get items from a single column from Station table, add number of occurences
    /// Supports columns with multiple values that are split by komma
    fn get_stations_multi_items(&self, column_name: &str) -> Result<HashMap<String, (u32,u32)>, Box<dyn Error>> {
        let mut items = HashMap::new();
        let mut my_stmt = self.pool
            .prepare(format!(
                "SELECT {column_name}, LastCheckOK FROM Station",
                column_name = column_name
            ))?;
        let result = my_stmt.execute(())?;

        for row in result {
            let (tags_str, ok): (String, bool) = mysql::from_row_opt(row?)?;
            let tags_arr = tags_str.split(',');
            for single_tag in tags_arr {
                let single_tag_trimmed = single_tag.trim().to_lowercase();
                if single_tag_trimmed != "" {
                    let counter = items.entry(single_tag_trimmed).or_insert((0,0));
                    counter.0 += 1;
                    if ok{
                        counter.1 += 1;
                    }
                }
            }
        }
        Ok(items)
    }

    /// Get currently cached items from table
    fn get_cached_items(
        &self,
        table_name: &str,
        column_name: &str,
    ) -> Result<HashMap<String, (u32, u32)>, Box<dyn Error>> {
        let mut items = HashMap::new();
        let mut my_stmt = self.pool
            .prepare(format!(
                "SELECT {column_name},StationCount, StationCountWorking FROM {table_name}",
                table_name = table_name,
                column_name = column_name
            ))?;
        let result = my_stmt.execute(())?;

        for row in result {
            let (key, value, value_working): (String, u32, u32) = mysql::from_row_opt(row?)?;
            let lower = key.to_lowercase();
            items.insert(lower, (value, value_working));
        }
        Ok(items)
    }

    fn update_cache_item(
        &self,
        tag: &String,
        count: u32,
        count_working: u32,
        table_name: &str,
        column_name: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut my_stmt = self.pool
            .prepare(format!(
                r"UPDATE {table_name} SET StationCount=?, StationCountWorking=? WHERE {column_name}=?",
                table_name = table_name,
                column_name = column_name
            ))?;
        let params = (count, count_working, tag);
        my_stmt.execute(params)?;
        Ok(())
    }

    fn insert_to_cache(&self, tags: HashMap<&String, (u32,u32)>, table_name: &str, column_name: &str) -> Result<(), Box<dyn Error>> {
        let query = format!(
            "INSERT INTO {table_name}({column_name},StationCount,StationCountWorking) VALUES(?,?,?)",
            table_name = table_name,
            column_name = column_name
        );
        let mut my_stmt = self.pool.prepare(query.trim_matches(','))?;
        for item in tags.iter() {
            my_stmt.execute((item.0, (item.1).0, (item.1).1))?;
        }
        Ok(())
    }

    fn remove_from_cache(&self, tags: Vec<&String>, table_name: &str, column_name: &str) -> Result<(), Box<dyn Error>> {
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
        let mut my_stmt = self.pool.prepare(query)?;
        my_stmt.execute(tags)?;
        Ok(())
    }

    fn vote_for_station(&self, ip: &str, station: Option<StationItem>) -> Result<String, Box<dyn Error>> {
        match station {
            Some(station) => {
                // delete ipcheck entries after 1 day minutes
                let query_1_delete = format!(r#"DELETE FROM IPVoteCheck WHERE TIME_TO_SEC(TIMEDIFF(Now(),VoteTimestamp))>24*60*60"#);
                let _result_1_delete = self.pool.prep_exec(query_1_delete, ())?;

                // was there a vote from the ip in the last 1 day?
                let query_2_vote_check = "SELECT StationID FROM IPVoteCheck WHERE StationID=:id AND IP=:ip";
                let result_2_vote_check = self.pool.prep_exec(query_2_vote_check, params!(ip, "id" => station.id))?;
                for resultsingle in result_2_vote_check {
                    for _ in resultsingle {
                        // do not allow vote
                        return Err(Box::new(DbError::VoteError("you are voting for the same station too often".to_string())));
                    }
                }

                // add vote entry
                let query_3_insert_votecheck = "INSERT INTO IPVoteCheck(IP,StationID,VoteTimestamp) VALUES(:ip,:id,UTC_TIMESTAMP())";
                let result_3_insert_votecheck =
                    self.pool.prep_exec(query_3_insert_votecheck, params!(ip,"id" => station.id))?;
                if result_3_insert_votecheck.affected_rows() == 0 {
                    return Err(Box::new(DbError::VoteError("could not insert vote check".to_string())));
                }

                // vote for station
                let query_4_update_votes = "UPDATE Station SET Votes=Votes+1 WHERE StationID=:id";
                let result_4_update_votes = self.pool.prep_exec(query_4_update_votes, params!("id" => station.id))?;
                if result_4_update_votes.affected_rows() == 1 {
                    Ok("voted for station successfully".to_string())
                } else {
                    Err(Box::new(DbError::VoteError("could not find station with matching id".to_string())))
                }
            }
            _ => Err(Box::new(DbError::VoteError("could not find station with matching id".to_string()))),
        }
    }

    fn increase_clicks(&self, ip: &str, station: &StationItem, seconds: u64) -> Result<bool,Box<dyn std::error::Error>> {
        let query = "SELECT StationUuid, IP FROM StationClick WHERE StationUuid=:stationuuid AND IP=ip AND TIME_TO_SEC(TIMEDIFF(UTC_TIMESTAMP(),ClickTimestamp))<:seconds";
        let result = self.pool.prep_exec(query, params!{"stationuuid" => &station.stationuuid, ip, seconds})?;

        for _ in result {
            return Ok(false);
        }

        let query2 = "INSERT INTO StationClick(IP,StationUuid,ClickUuid,ClickTimestamp,InsertTime) VALUES(:ip,:stationuuid,UUID(),UTC_TIMESTAMP(),UTC_TIMESTAMP())";
        let result2 = self.pool.prep_exec(query2, params!{"stationuuid" => &station.stationuuid, "ip" => ip})?;

        let query3 = "UPDATE Station SET ClickTimestamp=UTC_TIMESTAMP() WHERE StationUuid=:stationuuid";
        let result3 = self.pool.prep_exec(query3, params!{"stationuuid" => &station.stationuuid})?;

        if result2.affected_rows() == 1 && result3.affected_rows() == 1 {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    fn sync_votes(&self, list: Vec<Station>) -> Result<(), Box<dyn Error>> {
        trace!("sync_votes() 1");
        let mut transaction = self.pool.start_transaction(false, None, None)?;
        // get current list of votes in database
        let mut stations_current: HashMap<String, i32> = HashMap::new();
        {
            let result = transaction.prep_exec("SELECT StationUuid,Votes FROM Station",())?;
            for row in result {
                let (stationuuid, votes): (String, i32) = mysql::from_row_opt(row?)?;
                stations_current.insert(stationuuid, votes);
            }
        }
        trace!("sync_votes() 2");
        // compare and search for changes
        let mut rows_to_update: Vec<(String,i32)> = vec![];
        for station in list {
            let entry = stations_current.remove_entry(&station.stationuuid);
            if let Some(entry) = entry {
                let (stationuuid, votes) = entry;
                if votes != station.votes {
                    rows_to_update.push((stationuuid, votes));
                }
            }
        }
        trace!("sync_votes() 3");
        // update changed votes
        {
            let mut stmt = transaction.prepare("UPDATE Station SET Votes=GREATEST(Votes,:votes) WHERE StationUuid=:stationuuid;")?;
            for (stationuuid, votes) in rows_to_update {
                stmt.execute(params!(votes, stationuuid))?;
            }
        }
        trace!("sync_votes() 4");
        transaction.commit()?;
        trace!("sync_votes() 5");
        Ok(())
    }
}

fn fix_multi_field(value: &str) -> String {
    let values: Vec<String> = value.split(",").map(|v| v.trim().to_lowercase().to_string()).collect();
    values.join(",")
}

fn filter_order(order: &str) -> &str {
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
        "codec" => "Codec",
        "bitrate" => "Bitrate",
        "lastcheckok" => "LastCheckOK",
        "lastchecktime" => "LastCheckTime",
        "clicktimestamp" => "ClickTimestamp",
        "clickcount" => "clickcount",
        "clicktrend" => "ClickTrend",
        "random" => "RAND()",
        _ => "Name",
    }
}

fn filter_order_1_n(order: &str) -> Result<&str, Box<dyn Error>> {
    match order {
        "name" => Ok("name"),
        "stationcount" => Ok("stationcount"),
        _ => Err(Box::new(DbError::IllegalOrderError(String::from(order)))),
    }
}