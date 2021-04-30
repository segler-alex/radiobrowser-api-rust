mod migrations;
mod simple_migrate;
mod conversions;

use mysql::from_row;
use url::Url;
use mysql::Params;
use std::collections::HashSet;
use crate::db::db_error::DbError;

use std;
use std::collections::HashMap;

use celes::Country;
use uuid::Uuid;
use crate::db::models::State;
use crate::db::models::ExtraInfo;
use crate::db::models::StationItem;
use crate::db::models::StationCheckItem;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationChangeItemNew;
use crate::db::models::StationClickItem;
use crate::db::models::StationClickItemNew;
use crate::db::models::StationHistoryItem;
use crate::db::models::StationCheckStepItem;
use crate::db::models::StationCheckStepItemNew;
use crate::api::data::Station;
use std::error::Error;
use crate::db::DbConnection;
use mysql;
use mysql::Row;
use mysql::QueryResult;
use mysql::Value;
use mysql::TxOpts;
use mysql::prelude::*;

#[derive(Clone)]
pub struct MysqlConnection {
    pool: mysql::Pool,
}

impl MysqlConnection {
    const COLUMNS: &'static str =
        "StationID,ChangeUuid,StationUuid,Name,Url,Homepage,Favicon,UrlCache,
    Tags,Country,CountryCode,Subcountry,Language,Votes,
    Creation,
    Date_Format(Creation,'%Y-%m-%d %H:%i:%s') AS CreationFormated,
    Codec,Bitrate,Hls,LastCheckOK,
    LastCheckTime,
    Date_Format(LastCheckTime,'%Y-%m-%d %H:%i:%s') AS LastCheckTimeFormated,
    LastCheckOkTime,
    Date_Format(LastCheckOkTime,'%Y-%m-%d %H:%i:%s') AS LastCheckOkTimeFormated,
    LastLocalCheckTime,
    Date_Format(LastLocalCheckTime,'%Y-%m-%d %H:%i:%s') AS LastLocalCheckTimeFormated,
    ClickTimestamp,
    Date_Format(ClickTimestamp,'%Y-%m-%d %H:%i:%s') AS ClickTimestampFormated,
    clickcount,ClickTrend,
    LanguageCodes,SslError,GeoLat,GeoLong";

    const COLUMNS_CHECK: &'static str =
        "CheckID, StationUuid, CheckUuid, Source, Codec, Bitrate, Hls, CheckOK,
    CheckTime,
    Date_Format(CheckTime,'%Y-%m-%d %H:%i:%s') AS CheckTimeFormated,
    UrlCache,
    MetainfoOverridesDatabase,Public,Name,
    Description,Tags,CountryCode,
    Homepage,Favicon,Loadbalancer,
    CountrySubdivisionCode,ServerSoftware,Sampling,LanguageCodes,TimingMs,SslError,
    GeoLat,GeoLong";

    const COLUMNS_CLICK: &'static str =
        "ClickID, StationUuid, ClickUuid, IP,
    ClickTimestamp,
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

    fn get_list_from_query_result<A>(&self, results: QueryResult<mysql::Binary>) -> Result<Vec<A>, Box<dyn Error>> where A: From<Row> {
        let mut list: Vec<A> = vec![];
        for result in results {
            let row = result?;
            list.push(row.into());
        }
        Ok(list)
    }

    fn get_stations_query(&self, query: String) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, ())?;
        self.get_list_from_query_result(results)
    }

    pub fn get_single_column_number(&self, query: &str) -> Result<u64,Box<dyn std::error::Error>> {
        let row: Option<Row> = self.pool.get_conn()?.query_first(query)?;
        if let Some(mut row) = row {
            let items: u64 = row.take_opt(0).unwrap_or(Ok(0))?;
            return Ok(items);
        }
        return Ok(0);
    }

    pub fn get_single_column_number_params(&self, query: &str, p: mysql::Params) -> Result<u64,Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let row: Option<Row> = conn.exec_first(query, p)?;
        if let Some(mut row) = row {
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
            let query = format!("INSERT INTO StationHistory(Name,Url,Homepage,Favicon,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid,GeoLat,GeoLong)
                                                     SELECT Name,Url,Homepage,Favicon,CountryCode,SubCountry,Language,Tags,Votes,Creation,StationUuid,ChangeUuid,GeoLat,GeoLong FROM Station WHERE StationUuid IN ({})", insert_query.join(","));
            transaction.exec_drop(query, insert_params)?;
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
        let result = transaction.exec_iter(format!("SELECT ChangeUuid FROM StationHistory WHERE ChangeUuid IN ({})", select_query.join(",")), select_params)?;

        let mut list_result = vec![];
        for row in result {
            let (changeuuid,) = mysql::from_row_opt(row?)?;
            list_result.push(changeuuid);
        }
        Ok(list_result)
    }

    fn insert_station_by_change_internal(transaction: &mut mysql::Transaction<'_>, stationchanges: &[StationChangeItemNew])
         -> Result<Vec<String>,Box<dyn std::error::Error>> {
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
                insert_query.push("(?,?,?,?,?,?,?,?,?,?,?,?,?,'',UTC_TIMESTAMP())");
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
                insert_params.push(change.geo_lat.clone().into());
                insert_params.push(change.geo_long.clone().into());
                list_ids.push(change.stationuuid.clone());
            }
            let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,CountryCode,Subcountry,Language,Tags,ChangeUuid,StationUuid,GeoLat,GeoLong, UrlCache, Creation) 
                                    VALUES{}", insert_query.join(","));
            transaction.exec_drop(query, insert_params)?;
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
    fn calc_country_field(&mut self) -> Result<(), Box<dyn Error>> {
        trace!("calc_country_field() 0");
        let mut transaction = self.pool.start_transaction(TxOpts::default())?;

        transaction.query_drop("UPDATE Station SET CountryCode=UPPER(CountryCode)")?;
        transaction.query_drop("UPDATE StationHistory SET CountryCode=UPPER(CountryCode)")?;

        trace!("calc_country_field() 1");

        let query_select = "SELECT DISTINCT(CountryCode) FROM Station";
        let result: Vec<String> = transaction.query(query_select)?;
        let list: Vec<Params> = result.iter()
            .map(|cc| { (String::from(cc), Country::from_alpha2(cc).map(|d| d.long_name).unwrap_or(String::from("")) ) })
            .map(|co| params!{"countrycode" => co.0, "country" => co.1})
            .collect();
        
        trace!("calc_country_field() 2");
        let query_update = "UPDATE Station SET Country=:country WHERE CountryCode=:countrycode";
        transaction.exec_batch(query_update, list)?;
        trace!("calc_country_field() 3");
        /*
        let query_select = "SELECT DISTINCT(CountryCode) FROM Station";
        let result: Vec<String> = transaction.query(query_select)?;
        for c in result {
            match Country::from_alpha2(&c) {
                Ok(_)=>{},
                Err(_)=>{
                    warn!("Unknown countrycode '{}'", c);
                }
            }
        }
        */
        transaction.commit()?;
        trace!("calc_country_field() 4");

        Ok(())
    }

    fn clean_urls(&self, table_name: &str, column_key: &str, column_url: &str, allow_empty: bool) -> Result<(), Box<dyn Error>> {
        trace!("cleanurls({},{},{}) started..", table_name, column_key, column_url);
        let mut conn = self.pool.get_conn()?;
        let result_sets: Vec<(String,String)> = conn.query_iter(format!("SELECT {column_key},{column_url} FROM {table_name}", column_key = column_key, column_url = column_url, table_name = table_name))?
            .filter_map(|row| ok_output(row.map(from_row::<(String, String)>),"Unable to decode row"))
            .filter_map(|r| {
                match fix_url(&r.1, allow_empty) {
                    Ok(url) => {
                        if !r.1.eq(&url){
                            // parse ok and url changed
                            // -> change row in DB
                            Some((r.0, url))
                        }else{
                            // parse ok and url did not chang
                            // -> do nothing
                            None
                        }
                    },
                    Err(e) => {
                        if allow_empty {
                            // parse failed and empty is allowed
                            // -> reset broken value to empty
                            warn!("url parse failed of '{}': {}", r.1, e);
                            Some((r.0, "".to_string()))
                        }else{
                            // parse failed and empty is not allowed
                            // -> make it an error
                            // -> TODO: should remove line from DB
                            error!("url parse failed of '{}': {}", r.1, e);
                            None
                        }
                    }
                }
            })
            .collect();
        trace!("cleanurls({},{},{}) todo {}", table_name, column_key, column_url, result_sets.len());
        conn.exec_batch(format!("UPDATE {table_name} SET {column_url}=:url WHERE {column_key}=:uuid", table_name = table_name, column_url = column_url, column_key = column_key),
            result_sets.iter().map(|(uuid,url)|params!{uuid,url}))?;
        
        trace!("cleanurls({},{},{}) finished", table_name, column_key, column_url);
        Ok(())
    }

    fn delete_old_checks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let delete_old_checks_history_query = "DELETE FROM StationCheckHistory WHERE CheckTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(delete_old_checks_history_query, params!(seconds))?;
        Ok(())
    }

    fn delete_old_clicks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let delete_old_clicks_query = "DELETE FROM StationClick WHERE ClickTimestamp < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(delete_old_clicks_query, params!(seconds))?;
        Ok(())
    }

    fn delete_removed_from_history(&mut self) -> Result<(), Box<dyn Error>> {
        let query = "DELETE h FROM StationHistory h LEFT JOIN Station s ON s.StationUuid=h.StationUuid WHERE s.Tags IS NULL;";
        let mut conn = self.pool.get_conn()?;
        conn.query_drop(query)?;
        Ok(())
    }

    fn delete_never_working(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let delete_never_working_query = "DELETE FROM Station WHERE LastCheckOkTime IS NULL AND Creation < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(delete_never_working_query, params!(seconds))?;
        Ok(())
    }

    fn delete_were_working(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let delete_were_working_query = "DELETE FROM Station WHERE LastCheckOK=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(delete_were_working_query, params!(seconds))?;
        Ok(())
    }

    fn remove_unused_ip_infos_from_stationclicks(&mut self, seconds: u64) -> Result<(), Box<dyn Error>> {
        let query = "UPDATE StationClick SET IP=NULL WHERE InsertTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(query, params!(seconds))?;
        Ok(())
    }

    fn remove_illegal_icon_links(&mut self) -> Result<(), Box<dyn Error>> {
        let query = r#"UPDATE Station SET Favicon="" WHERE LOWER(Favicon) NOT LIKE 'http://%' AND LOWER(Favicon) NOT LIKE'https://%' AND Favicon<>"";"#;
        let mut conn = self.pool.get_conn()?;
        conn.query_drop(query)?;
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
        self.pool.get_conn()?.query_drop(query)?;
        trace!("update_stations_clickcount() 2");
        Ok(())
    }

    fn get_station_count_broken(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOK=0 OR LastCheckOK IS NULL")
    }

    fn get_station_count_working(&self) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOK=1")
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
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, ())?;
        self.get_list_from_query_result(results)
    }

    fn get_station_by_uuid(&self, id_str: &str) -> Result<Vec<StationItem>,Box<dyn Error>> {
        let query = format!(
            "SELECT {columns} from Station WHERE StationUuid=? ORDER BY Name",
            columns = MysqlConnection::COLUMNS
        );
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, (id_str,))?;
        self.get_list_from_query_result(results)
    }

    fn get_stations_by_uuid(&self, uuids: Vec<String>) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let search_params: Vec<Value> = uuids.iter().map(|item| item.clone().into()).collect();
        let search_query: Vec<&str> = (0..search_params.len()).map(|_item| "?").collect();

        if search_query.len() > 0 {
            let query_select_stations_by_uuid = format!("SELECT {columns} FROM Station WHERE StationUuid IN ({items})", items=search_query.join(","),columns=MysqlConnection::COLUMNS);
            let mut conn = self.pool.get_conn()?;
            let result = conn.exec_iter(query_select_stations_by_uuid, search_params)?;
            self.get_list_from_query_result(result)
        }else{
            Ok(vec![])
        }
    }

    fn get_deletable_never_working(&self, seconds: u64) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOkTime IS NULL AND Creation < UTC_TIMESTAMP() - INTERVAL :seconds SECOND", params!(seconds))
    }

    fn get_deletable_were_working(&self, seconds: u64) -> Result<u64, Box<dyn Error>> {
        self.get_single_column_number_params("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOK=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND", params!(seconds))
    }

    fn get_stations_broken(&self, offset: u32, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        self.get_stations_query(format!(
            "SELECT {columns} from Station WHERE LastCheckOK=FALSE ORDER BY rand() LIMIT {offset},{limit}",
            columns = MysqlConnection::COLUMNS,
            offset = offset,
            limit = limit
        ))
    }

    fn get_stations_topvote(&self, hidebroken: bool, offset: u32, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        let hidebroken_string = if hidebroken {
            " WHERE LastCheckOK=TRUE"
        } else {
            ""
        };
        query = format!(
            "SELECT {columns} from Station {where} ORDER BY Votes DESC LIMIT {offset},{limit}",
            columns = MysqlConnection::COLUMNS,
            where = hidebroken_string,
            offset = offset,
            limit = limit
        );
        self.get_stations_query(query)
    }

    fn get_stations_topclick(&self, hidebroken: bool, offset: u32, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        let hidebroken_string = if hidebroken {
            " WHERE LastCheckOK=TRUE"
        } else {
            ""
        };
        query = format!(
            "SELECT {columns} from Station {where} ORDER BY clickcount DESC LIMIT {offset},{limit}",
            columns = MysqlConnection::COLUMNS,
            where = hidebroken_string,
            offset = offset,
            limit = limit
        );
        self.get_stations_query(query)
    }

    fn get_stations_lastclick(&self, hidebroken: bool, offset: u32, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        let hidebroken_string = if hidebroken {
            " WHERE LastCheckOK=TRUE"
        } else {
            ""
        };
        query = format!(
            "SELECT {columns} from Station {where} ORDER BY ClickTimestamp DESC LIMIT {offset},{limit}",
            columns = MysqlConnection::COLUMNS,
            where = hidebroken_string,
            offset = offset,
            limit = limit
        );
        self.get_stations_query(query)
    }

    fn get_stations_lastchange(&self, hidebroken: bool, offset: u32, limit: u32) -> Result<Vec<StationItem>, Box<dyn Error>> {
        let query: String;
        let hidebroken_string = if hidebroken {
            " WHERE LastCheckOK=TRUE"
        } else {
            ""
        };
        query = format!(
            "SELECT {columns} from Station {where} ORDER BY Creation DESC LIMIT {offset},{limit}",
            columns = MysqlConnection::COLUMNS,
            where = hidebroken_string,
            offset = offset,
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
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, (search.to_lowercase(),))?;
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
        let mut conn = self.pool.get_conn()?;
        let results = if exact {
            conn.exec_iter(query, (&search, &search, &search, &search))?
        } else {
            conn.exec_iter(query, (search,))?
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
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, ())?;
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
        has_geo_info: Option<bool>,
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
        match has_geo_info {
            Some(has_geo_info) => {
                if has_geo_info {
                    query.push_str(" AND GeoLat IS NOT NULL AND GeoLong IS NOT NULL");
                }else{
                    query.push_str(" AND (GeoLat IS NULL OR GeoLong IS NULL)");
                }
            },
            None => {}
        }
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
            query.push_str(" AND LOWER(Codec)=LOWER(:codec)");
        }
        let mut params: Vec<(String,Value)> = vec!{
            (String::from("name"), Value::from(name.unwrap_or_default())),
            (String::from("country"), Value::from(country.unwrap_or_default())),
            (String::from("countrycode"), Value::from(countrycode.unwrap_or_default())),
            (String::from("state"), Value::from(state.unwrap_or_default())),
            (String::from("language"), Value::from(language.unwrap_or_default())),
            (String::from("tag"), Value::from(tag.unwrap_or_default())),
            (String::from("codec"), Value::from(codec.unwrap_or_default())),
            (String::from("bitrate_min"), Value::from(bitrate_min)),
            (String::from("bitrate_max"), Value::from(bitrate_max)),
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
        
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(
            query,
            params,
        )?;
        self.get_list_from_query_result(results)
    }

    fn get_changes(&self, stationuuid: Option<String>, changeuuid: Option<String>, limit: u32) -> Result<Vec<StationHistoryItem>, Box<dyn Error>> {
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
                Subcountry,
                CountryCode,
                Language,Votes,
                LanguageCodes,
                GeoLat,GeoLong,
                Creation,
                Date_Format(Creation,'%Y-%m-%d %H:%i:%s') AS CreationFormated
                from StationHistory WHERE 1=:mynumber {changeuuid_str} {stationuuid} ORDER BY StationChangeID ASC LIMIT {limit}", changeuuid_str = changeuuid_str, stationuuid = stationuuid_str, limit = limit);
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, params! {
            "mynumber" => 1,
            "stationuuid" => stationuuid.unwrap_or(String::from("")),
            "changeuuid" => changeuuid.unwrap_or(String::from(""))
        })?;
        self.get_list_from_query_result(results)
    }

    /// Select all historic changes for stations with the given uuids
    /// ordered by creation date
    fn get_changes_for_stations(&self, mut station_uuids: Vec<String>) -> Result<Vec<StationHistoryItem>, Box<dyn Error>> {
        if station_uuids.len() > 0{
            let stationuuids_query: Vec<&str> = (0..station_uuids.len()).map(|_item| "?").collect();
            let stationuuids_str = stationuuids_query.join(",");

            let stationuuids_params: Vec<Value> = station_uuids.drain(..).map(|item| item.into()).collect();
            
            let query: String = format!("SELECT StationChangeID,ChangeUuid,
                    StationUuid,Name,
                    Url,Homepage,
                    Favicon,Tags,
                    Subcountry,
                    CountryCode,
                    Language,Votes,
                    LanguageCodes,
                    GeoLat,GeoLong,
                    Creation,
                    Date_Format(Creation,'%Y-%m-%d %H:%i:%s') AS CreationFormated
                    from StationHistory WHERE StationUuid IN ({stationuuids_str}) ORDER BY Creation ASC", stationuuids_str = stationuuids_str);
            let mut conn = self.pool.get_conn()?;
            let results = conn.exec_iter(query, stationuuids_params)?;
            self.get_list_from_query_result(results)
        } else {
            Ok(vec![])
        }
    }

    fn add_station_opt(&self, name: Option<String>, url: Option<String>, homepage: Option<String>, favicon: Option<String>,
        countrycode: Option<String>, state: Option<String>, language: Option<String>, languagecodes: Option<String>, tags: Option<String>, geo_lat: Option<f64>, geo_long: Option<f64>) -> Result<String, Box<dyn Error>> {
        let mut transaction = self.pool.start_transaction(TxOpts::default())?;

        let countrycode: String = countrycode.unwrap_or_default().to_uppercase();
        let country: String = Country::from_alpha2(&countrycode).map(|c| c.long_name).unwrap_or(String::from(""));

        let query = format!("INSERT INTO Station(Name,Url,Homepage,Favicon,Country,CountryCode,Subcountry,Language,LanguageCodes,Tags,ChangeUuid,StationUuid,GeoLat,GeoLong,UrlCache,Creation) 
                        VALUES(:name, :url, :homepage, :favicon, :country, :countrycode, :state, :language, :languagecodes, :tags, :changeuuid, :stationuuid, :geo_lat, :geo_long, '', UTC_TIMESTAMP())");

        let name = name.ok_or(DbError::AddStationError(String::from("name is empty")))?;
        let url = url.map(|x|fix_url(&x, false)).transpose()?;
        let homepage = homepage.map(|x|fix_url(&x, true)).transpose()?;

        if countrycode.len() != 2 {
            return Err(Box::new(DbError::AddStationError(String::from("countrycode does not have exactly 2 chars"))));
        }

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
            "country" => country,
            "countrycode" => countrycode,
            "state" => state.unwrap_or_default(),
            "language" => fix_multi_field(&language.unwrap_or_default()),
            "languagecodes" => fix_multi_field(&languagecodes.unwrap_or_default()),
            "tags" => fix_multi_field(&tags.unwrap_or_default()),
            "changeuuid" => changeuuid,
            "stationuuid" => stationuuid.clone(),
            "geo_lat" => geo_lat,
            "geo_long" => geo_long,
        };

        transaction.exec_drop(query, params)?;
        MysqlConnection::backup_stations_by_uuid(&mut transaction, &(vec![stationuuid.clone()]))?;
        transaction.commit()?;

        Ok(stationuuid)
    }

    fn get_pull_server_lastid(&self, server: &str) -> Result<Option<String>, Box<dyn Error>> {
        let query: String = format!("SELECT lastid FROM PullServers WHERE name=:name");
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, params!{
            "name" => server
        });
        match results {
            Ok(results) => {
                for result in results {
                    if let Ok(mut result) = result {
                        let lastid = result.take_opt("lastid");
                        if let Some(lastid) = lastid {
                            if let Ok(lastid) = lastid {
                                return Ok(Some(lastid));
                            }
                        }
                    }
                };
                Ok(None)
            },
            _ => Ok(None)
        }
    }

    fn set_pull_server_lastid(&self, server: &str, lastid: &str) -> Result<(),Box<dyn std::error::Error>> {
        let params = params!{
            "name" => server,
            "lastid" => lastid,
        };
        let mut conn = self.pool.get_conn()?;
        let query_update: String = format!("UPDATE PullServers SET lastid=:lastid WHERE name=:name");
        let results_update = conn.exec_iter(query_update, &params)?.affected_rows();
        if results_update == 0 {
            let query_insert: String = format!("INSERT INTO PullServers(name, lastid) VALUES(:name,:lastid)");
            conn.exec_drop(query_insert, &params)?;
        }
        Ok(())
    }

    fn get_pull_server_lastcheckid(&self, server: &str) -> Result<Option<String>, Box<dyn Error>> {
        let query: String = format!("SELECT lastcheckid FROM PullServers WHERE name=:name");
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, params!{
            "name" => server
        });
        match results {
            Ok(results) => {
                for result in results {
                    if let Ok(mut result) = result {
                        let lastcheckid = result.take_opt("lastcheckid");
                        if let Some(lastcheckid) = lastcheckid {
                            if let Ok(lastcheckid) = lastcheckid {
                                return Ok(Some(lastcheckid));
                            }
                        }
                    }
                };
                Ok(None)
            },
            _ => Ok(None)
        }
    }

    fn set_pull_server_lastcheckid(&self, server: &str, lastcheckid: &str) -> Result<(),Box<dyn std::error::Error>> {
        let params = params!{
            "name" => server,
            "lastcheckid" => lastcheckid,
        };
        let mut conn = self.pool.get_conn()?;
        let query_update: String = format!("UPDATE PullServers SET lastcheckid=:lastcheckid WHERE name=:name");
        let results_update = conn.exec_iter(query_update, &params)?.affected_rows();
        if results_update == 0 {
            let query_insert: String = format!("INSERT INTO PullServers(name, lastcheckid) VALUES(:name,:lastcheckid)");
            conn.exec_drop(query_insert, &params)?;
        }
        Ok(())
    }


    fn get_pull_server_lastclickid(&self, server: &str) -> Result<Option<String>, Box<dyn Error>> {
        let query: String = format!("SELECT lastclickuuid FROM PullServers WHERE name=:name");
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, params!{
            "name" => server
        });
        match results {
            Ok(results) => {
                for result in results {
                    if let Ok(mut result) = result {
                        let lastclickuuid = result.take_opt("lastclickuuid");
                        if let Some(lastclickuuid) = lastclickuuid {
                            if let Ok(lastclickuuid) = lastclickuuid {
                                return Ok(Some(lastclickuuid));
                            }
                        }
                    }
                };
                Ok(None)
            },
            _ => Ok(None)
        }
    }

    fn set_pull_server_lastclickid(&self, server: &str, lastclickuuid: &str) -> Result<(),Box<dyn std::error::Error>> {
        let params = params!{
            "name" => server,
            "lastclickuuid" => lastclickuuid,
        };
        let mut conn = self.pool.get_conn()?;
        let query_update: String = format!("UPDATE PullServers SET lastclickuuid=:lastclickuuid WHERE name=:name");
        let results_update = conn.exec_iter(query_update, &params)?.affected_rows();
        if results_update == 0 {
            let query_insert: String = format!("INSERT INTO PullServers(name, lastclickuuid) VALUES(:name,:lastclickuuid)");
            conn.exec_drop(query_insert, &params)?;
        }
        Ok(())
    }

    fn insert_station_by_change(&self, list_station_changes: &[StationChangeItemNew]) -> Result<Vec<String>,Box<dyn std::error::Error>> {
        let mut transaction = self.pool.start_transaction(TxOpts::default())?;

        let list_ids = MysqlConnection::insert_station_by_change_internal(&mut transaction, list_station_changes)?;
        MysqlConnection::backup_stations_by_uuid(&mut transaction, &list_ids)?;

        transaction.commit()?;
        Ok(list_ids)
    }

    fn insert_checks(&self, list: Vec<StationCheckItemNew>) -> Result<(Vec<StationCheckItemNew>,Vec<StationCheckItemNew>,Vec<StationCheckItemNew>), Box<dyn std::error::Error>> {
        trace!("insert_checks()");
        let mut transaction = self.pool.start_transaction(TxOpts::default())?;
        
        // search for checkuuids in history table, if already added (maybe from other source)
        let mut existing_checks_uuids: HashSet<String> = HashSet::new();
        {
            let search_params: Vec<Value> = list.iter().filter_map(|item| item.checkuuid.clone()).map(|item2| item2.into()).collect();
            let search_query: Vec<&str> = (0..search_params.len()).map(|_item| "?").collect();

            if search_query.len() > 0 {
                let query_delete_old_station_checks = format!("SELECT CheckUuid FROM StationCheckHistory WHERE CheckUuid IN ({})", search_query.join(","));
                let result = transaction.exec_iter(query_delete_old_station_checks, search_params)?;

                for row in result {
                    let (checkuuid, ) = mysql::from_row_opt(row?)?;
                    existing_checks_uuids.replace(checkuuid);
                }
            }
        }

        trace!("Ignored checks(already existing) for insert: {}", existing_checks_uuids.len());

        // search for stations by stationuuid
        let mut existing_stations: HashSet<String> = HashSet::new();
        {
            let search_params: Vec<Value> = list.iter().map(|item| item.station_uuid.clone().into()).collect();
            let search_query: Vec<&str> = (0..search_params.len()).map(|_item| "?").collect();

            if search_query.len() > 0 {
                let query_select_stations_by_uuid = format!("SELECT StationUuid FROM Station WHERE StationUuid IN ({})", search_query.join(","));
                let result = transaction.exec_iter(query_select_stations_by_uuid, search_params)?;

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
        let mut inserted: Vec<StationCheckItemNew> = vec![];
        let mut ignored_checks_no_station: Vec<StationCheckItemNew> = vec![];
        let mut existing_checks: Vec<StationCheckItemNew> = vec![];
        for item in list {
            // ignore checks, where there is no station in the database
            if !existing_stations.contains(&item.station_uuid) {
                //ignored_checks_no_station.replace(item.station_uuid.clone());
                ignored_checks_no_station.push(item);
                continue;
            }
            // check has checkuuid ?
            match &item.checkuuid {
                Some(checkuuid) => {
                    // ignore checks that are already in the database
                    if existing_checks_uuids.contains(checkuuid) {
                        existing_checks.push(item);
                        continue;
                    }
                    // reuse checkuuid
                    match &item.timestamp {
                        Some(timestamp) => {
                            insert_station_check_query.push("(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP(),?,?,?,?,?)");
                            insert_station_check_params.push(checkuuid.into());
                            insert_station_check_params.push(timestamp.into());
                        }
                        None => {
                            insert_station_check_query.push("(?,UTC_TIMESTAMP(),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP(),?,?,?,?,?)");
                            insert_station_check_params.push(checkuuid.into());
                        }
                    }
                },
                None => {
                    // generate new checkuuid
                    match &item.timestamp {
                        Some(timestamp) => {
                            insert_station_check_query.push("(UUID(),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP(),?,?,?,?,?)");
                            insert_station_check_params.push(timestamp.into());
                        }
                        None => {
                            insert_station_check_query.push("(UUID(),UTC_TIMESTAMP(),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,UTC_TIMESTAMP(),?,?,?,?,?)");
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
            insert_station_check_params.push(item.do_not_index.clone().into());
            
            insert_station_check_params.push(item.server_software.clone().into());
            insert_station_check_params.push(item.sampling.clone().into());
            insert_station_check_params.push(item.languagecodes.clone().into());
            insert_station_check_params.push(item.timing_ms.clone().into());
            insert_station_check_params.push(item.countrysubdivisioncode.clone().into());

            inserted.push(item);
        }

        trace!("Ignored checks(no stations) for insert: {}", ignored_checks_no_station.len());

        // insert into history table
        if insert_station_check_query.len() > 0 {
            let insert_station_check_params_str = insert_station_check_query.join(",");
            let query_insert_station_check_history = format!("INSERT INTO StationCheckHistory(CheckUuid,CheckTime,StationUuid,Source,Codec,Bitrate,Hls,CheckOK,UrlCache,
                MetainfoOverridesDatabase,Public,Name,Description,Tags,CountryCode,Homepage,Favicon,Loadbalancer,DoNotIndex,InsertTime,ServerSoftware,Sampling,LanguageCodes,TimingMs,CountrySubdivisionCode) VALUES{}", insert_station_check_params_str);
            transaction.exec_drop(query_insert_station_check_history, insert_station_check_params)?;
        }

        transaction.commit()?;

        Ok((existing_checks,ignored_checks_no_station,inserted))
    }

    /// Select all checks that are currently in the database of a station with the given uuid
    /// and calculate an overall status by majority vote. Ties are broken with the own vote
    /// of the most current check
    fn update_station_with_check_data(&self, list: &Vec<StationCheckItemNew>, local: bool) -> Result<(), Box<dyn std::error::Error>> {
        trace!("update_station_with_check_data()");
        let mut transaction = self.pool.start_transaction(TxOpts::default())?;

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
            let result = transaction.exec_iter(
                format!("SELECT StationUuid,ROUND(AVG(CheckOk)) AS result FROM StationCheck WHERE StationUuid IN ({}) GROUP BY StationUuid", uuids = query_in),
                list_station_uuid
            )?;

            for row in result {
                let (stationuuid, result): (String, u8,) = mysql::from_row_opt(row?)?;
                majority_vote.insert(stationuuid, result == 1);
            }
        }

        {
            for item in list {
                let vote = majority_vote.get(&item.station_uuid).unwrap_or(&true);

                let mut params: Vec<(String, Value)> = vec!{
                    (String::from("one"), Value::from(1)),
                    (String::from("codec"), Value::from(&item.codec)),
                    (String::from("bitrate"), Value::from(item.bitrate)),
                    (String::from("hls"), Value::from(item.hls)),
                    (String::from("stationuuid"), Value::from(&item.station_uuid)),
                    (String::from("vote"), Value::from(vote)),
                    (String::from("ssl_error"), Value::from(item.ssl_error)),
                };

                if item.metainfo_overrides_database {
                    let mut query = vec![];
                    let do_not_index = item.do_not_index.unwrap_or(false);
                    if !do_not_index {
                        debug!("override station: uuid='{}'", item.station_uuid);

                        params.push((String::from("urlcache"), item.url.clone().into(),));

                        if let Some(name) = &item.name {
                            params.push((String::from("name"),name.into(),));
                            query.push("Name=:name");
                        }
                        if let Some(homepage) = &item.homepage {
                            params.push((String::from("homepage"),homepage.into(),));
                            query.push("Homepage=:homepage");
                        }
                        if let Some(loadbalancer) = &item.loadbalancer {
                            params.push((String::from("loadbalancer"),loadbalancer.into(),));
                            query.push("Url=:loadbalancer");
                        }
                        if let Some(countrycode) = &item.countrycode {
                            params.push((String::from("countrycode"),countrycode.into(),));
                            query.push("CountryCode=UPPER(:countrycode)");
                        }
                        if let Some(tags) = &item.tags {
                            params.push((String::from("tags"),fix_multi_field(tags).into(),));
                            query.push("Tags=:tags");
                        }
                        if let Some(favicon) = &item.favicon {
                            params.push((String::from("favicon"),favicon.into(),));
                            query.push("Favicon=:favicon");
                        }
                        if let Some(geo_lat) = &item.geo_lat {
                            params.push((String::from("geo_lat"),geo_lat.into(),));
                            query.push("GeoLat=:geo_lat");
                        }
                        if let Some(geo_long) = &item.geo_long {
                            params.push((String::from("geo_long"),geo_long.into(),));
                            query.push("GeoLong=:geo_long");
                        }
                        query.push("SslError=:ssl_error");
                        query.push("LastCheckOk=:vote");
                        if local {
                            query.push("LastLocalCheckTime=UTC_TIMESTAMP()");
                        }

                        if item.check_ok {
                            let query_update_ok = format!("UPDATE Station SET LastCheckOkTime=UTC_TIMESTAMP(),LastCheckTime=UTC_TIMESTAMP(),Codec=:codec,Bitrate=:bitrate,Hls=:hls,UrlCache=:urlcache,{} WHERE StationUuid=:stationuuid", query.join(","));
                            transaction.exec_drop(query_update_ok, params)?;
                        }
                    }else{
                        let query_delete = "DELETE FROM Station WHERE StationUuid=:stationuuid";
                        transaction.exec_drop(query_delete, params)?;
                    }
                }else{
                    if item.check_ok {
                        params.push((String::from("urlcache"), item.url.clone().into(),));

                        let query_update_ok = format!("UPDATE Station SET {lastlocalchecktime}LastCheckOkTime=UTC_TIMESTAMP(),LastCheckTime=UTC_TIMESTAMP(),Codec=:codec,Bitrate=:bitrate,Hls=:hls,UrlCache=:urlcache,LastCheckOk=:vote,SslError=:ssl_error WHERE StationUuid=:stationuuid",
                            lastlocalchecktime = if local {"LastLocalCheckTime=UTC_TIMESTAMP(),"} else {""},
                        );
                        transaction.exec_drop(query_update_ok, params)?;
                    }else{
                        let query_update_check_ok = format!("UPDATE Station st SET {lastlocalchecktime}LastCheckTime=UTC_TIMESTAMP(),LastCheckOk=:vote WHERE StationUuid=:stationuuid",
                            lastlocalchecktime = if local {"LastLocalCheckTime=UTC_TIMESTAMP(),"} else {""},
                        );
                        transaction.exec_drop(query_update_check_ok, params)?;
                    }
                }
            }
        }
        transaction.commit()?;

        Ok(())
    }

    fn insert_clicks(&self, list: &Vec<StationClickItemNew>) -> Result<(), Box<dyn Error>> {
        let mut transaction = self.pool.start_transaction(TxOpts::default())?;

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
                let result = transaction.exec_iter(query, search_click_params)?;
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
                let result = transaction.exec_iter(query, search_station_params)?;
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
            transaction.exec_drop(query, insert_click_params)?;
        }

        transaction.commit()?;

        Ok(())
    }

    fn get_checks(&self, stationuuid: Option<String>, checkuuid: Option<String>, seconds: u32, include_history: bool, limit: u32) -> Result<Vec<StationCheckItem>, Box<dyn Error>> {
        let table_name = if include_history { "StationCheckHistory" } else { "StationCheck" };
        let where_seconds = if seconds > 0 {
            format!(
                "AND TIMESTAMPDIFF(SECOND,CheckTime,UTC_TIMESTAMP())<{seconds}",
                seconds = seconds
            )
        } else {
            String::from("")
        };

        let mut query_params: Vec<(String, Value)> = vec!{(String::from("one"), Value::from(1))};
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
                format!("SELECT {columns} FROM {table_name} WHERE StationUuid=:stationuuid {where_checkuuid} {where_seconds} ORDER BY CheckID LIMIT {limit}", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str, table_name = table_name, limit = limit)
            }
            None => {
                format!("SELECT {columns} FROM {table_name} WHERE 1=:one {where_checkuuid} {where_seconds} ORDER BY CheckID LIMIT {limit}", columns = MysqlConnection::COLUMNS_CHECK, where_seconds = where_seconds, where_checkuuid = where_checkuuid_str, table_name = table_name, limit = limit)
            }
        };

        trace!("get_checks() {}", query);
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, query_params)?;

        self.get_list_from_query_result(results)
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

        let mut query_params: Vec<(String, Value)> = vec!{(String::from("one"), Value::from(1))};
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
        let mut conn = self.pool.get_conn()?;
        let results = conn.exec_iter(query, query_params)?;

        self.get_list_from_query_result(results)
    }

    fn get_extra(
        &self,
        table_name: &str,
        column_name: &str,
        search: Option<String>,
        order: String,
        reverse: bool,
        hidebroken: bool,
        offset: u32,
        limit: u32,
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
        let mut conn = self.pool.get_conn()?;
        let result = conn.exec_iter(format!("SELECT {column_name} AS name, {hidebroken} FROM {table_name} WHERE {column_name} <> '' {search} HAVING stationcount > 0 ORDER BY {order} {reverse} LIMIT {offset},{limit}",
            search = search_string, order = order,
            reverse = reverse_string, hidebroken = hidebroken_string,
            table_name = table_name, column_name = column_name,
            offset = offset, limit = limit), params)?;
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
        offset: u32,
        limit: u32,
    ) -> Result<Vec<ExtraInfo>, Box<dyn Error>> {
        let order = filter_order_1_n(&order)?;
        let query: String;
        let reverse_string = if reverse { "DESC" } else { "ASC" };
        let hidebroken_string = if hidebroken {
            " AND LastCheckOK=TRUE"
        } else {
            ""
        };
        let mut conn = self.pool.get_conn()?;
        let result = match search {
            Some(value) => {
                query = format!("SELECT {column} AS name,COUNT(*) AS stationcount FROM Station WHERE UPPER({column}) LIKE UPPER(CONCAT('%',?,'%')) AND {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse} LIMIT {offset},{limit}",
                    column = column, order = order, reverse = reverse_string,
                    hidebroken = hidebroken_string,
                    offset = offset,
                    limit = limit,
                );
                conn.exec_iter(query, (value,))
            }
            None => {
                query = format!("SELECT {column} AS name,COUNT(*) AS stationcount FROM Station WHERE {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse} LIMIT {offset},{limit}",
                    column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string,
                    offset = offset,
                    limit = limit,
                );
                conn.exec_iter(query, ())
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
        offset: u32,
        limit: u32,
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

        let mut conn = self.pool.get_conn()?;
        let result = conn.exec_iter(format!(r"SELECT Subcountry AS name,Country,COUNT(*) AS stationcount FROM Station WHERE Subcountry <> '' {country} {search} {hidebroken} GROUP BY Subcountry, Country ORDER BY {order} {reverse} LIMIT {offset},{limit}",
        hidebroken = hidebroken_string, order = order, country = country_string, reverse = reverse_string, search = search_string, limit = limit, offset = offset), params)?;
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
        let mut conn = self.pool.get_conn()?;
        let result = conn
            .exec_iter(format!(
                "SELECT {column_name}, LastCheckOK FROM Station",
                column_name = column_name
            ), ())?;

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
        let mut conn = self.pool.get_conn()?;
        let result = conn
            .exec_iter(format!(
                "SELECT {column_name},StationCount, StationCountWorking FROM {table_name}",
                table_name = table_name,
                column_name = column_name
            ), ())?;

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
        let query = format!(
            r"UPDATE {table_name} SET StationCount=?, StationCountWorking=? WHERE {column_name}=?",
            table_name = table_name,
            column_name = column_name
        );
        self.pool.get_conn()?.exec_drop(query, (count, count_working, tag))?;
        Ok(())
    }

    fn insert_to_cache(&self, tags: HashMap<&String, (u32,u32)>, table_name: &str, column_name: &str) -> Result<(), Box<dyn Error>> {
        let query = format!(
            r"INSERT INTO {table_name}({column_name},StationCount,StationCountWorking) VALUES(?,?,?)",
            table_name = table_name,
            column_name = column_name
        );
        self.pool.get_conn()?.exec_batch(query, tags.iter().map(|item| (item.0, (item.1).0, (item.1).1)))?;
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
        self.pool.get_conn()?.exec_drop(query, tags)?;
        Ok(())
    }

    fn vote_for_station(&self, ip: &str, station: Option<StationItem>) -> Result<String, Box<dyn Error>> {
        match station {
            Some(station) => {
                let mut conn = self.pool.get_conn()?;
                // delete ipcheck entries after 1 day minutes
                let query_1_delete = format!(r#"DELETE FROM IPVoteCheck WHERE TIME_TO_SEC(TIMEDIFF(UTC_TIMESTAMP,VoteTimestamp))>24*60*60"#);
                conn.exec_drop(query_1_delete, ())?;

                // was there a vote from the ip in the last 1 day?
                let query_2_vote_check = "SELECT StationID FROM IPVoteCheck WHERE StationID=:id AND IP=:ip";
                let result_2_vote_check = conn.exec_iter(query_2_vote_check, params!(ip, "id" => station.id))?;
                for resultsingle in result_2_vote_check {
                    for _ in resultsingle {
                        // do not allow vote
                        return Err(Box::new(DbError::VoteError("you are voting for the same station too often".to_string())));
                    }
                }

                // add vote entry
                let query_3_insert_votecheck = "INSERT INTO IPVoteCheck(IP,StationID,VoteTimestamp) VALUES(:ip,:id,UTC_TIMESTAMP())";
                let result_3_insert_votecheck =
                    conn.exec_iter(query_3_insert_votecheck, params!(ip,"id" => station.id))?.affected_rows();
                if result_3_insert_votecheck == 0 {
                    return Err(Box::new(DbError::VoteError("could not insert vote check".to_string())));
                }

                // vote for station
                let query_4_update_votes = "UPDATE Station SET Votes=Votes+1 WHERE StationID=:id";
                let result_4_update_votes = conn.exec_iter(query_4_update_votes, params!("id" => station.id))?.affected_rows();
                if result_4_update_votes == 1 {
                    Ok("voted for station successfully".to_string())
                } else {
                    Err(Box::new(DbError::VoteError("could not find station with matching id".to_string())))
                }
            }
            _ => Err(Box::new(DbError::VoteError("could not find station with matching id".to_string()))),
        }
    }

    fn increase_clicks(&self, ip: &str, station: &StationItem, seconds: u64) -> Result<bool,Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let query = "SELECT StationUuid, IP FROM StationClick WHERE StationUuid=:stationuuid AND IP=:ip AND TIME_TO_SEC(TIMEDIFF(UTC_TIMESTAMP(),ClickTimestamp))<:seconds";
        let result = conn.exec_iter(query, params!{"stationuuid" => &station.stationuuid, ip, seconds})?;

        for _ in result {
            return Ok(false);
        }

        let query2 = "INSERT INTO StationClick(IP,StationUuid,ClickUuid,ClickTimestamp,InsertTime) VALUES(:ip,:stationuuid,UUID(),UTC_TIMESTAMP(),UTC_TIMESTAMP())";
        let result2 = conn.exec_iter(query2, params!{"stationuuid" => &station.stationuuid, "ip" => ip})?.affected_rows();

        let query3 = "UPDATE Station SET ClickTimestamp=UTC_TIMESTAMP() WHERE StationUuid=:stationuuid";
        let result3 = conn.exec_iter(query3, params!{"stationuuid" => &station.stationuuid})?.affected_rows();

        if result2 == 1 && result3 == 1 {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    fn sync_votes(&self, list: Vec<Station>) -> Result<(), Box<dyn Error>> {
        trace!("sync_votes() 1");
        let mut transaction = self.pool.start_transaction(TxOpts::default())?;
        // get current list of votes in database
        let mut stations_current: HashMap<String, i32> = HashMap::new();
        {
            let result = transaction.exec_iter("SELECT StationUuid,Votes FROM Station",())?;
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
                    rows_to_update.push((stationuuid, station.votes));
                }
            }
        }
        trace!("sync_votes() 3");
        // update changed votes
        {
            transaction.exec_batch("UPDATE Station SET Votes=GREATEST(Votes,:votes) WHERE StationUuid=:stationuuid;", rows_to_update.iter().map(|(stationuuid, votes)| params!(votes, stationuuid)))?;
        }
        trace!("sync_votes() 4");
        transaction.commit()?;
        trace!("sync_votes() 5");
        Ok(())
    }

    fn insert_station_check_steps(&mut self, station_check_steps: &[StationCheckStepItemNew])
    -> Result<(),Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_batch(
            r"INSERT INTO StationCheckStep (StationUuid,CheckUuid,Url,UrlType,Error,StepUuid,ParentStepUuid,InsertTime)
              VALUES (:stationuuid, :checkuuid, :url, :urltype, :error, :stepuuid, :parentstepuuid, UTC_TIMESTAMP())",
            station_check_steps.iter().map(|p| params! {
                "stationuuid" => &p.stationuuid,
                "checkuuid" => &p.checkuuid,
                "stepuuid" => &p.stepuuid,
                "parentstepuuid" => &p.parent_stepuuid,
                "url" => &p.url,
                "urltype" => &p.urltype,
                "error" => &p.error,
            })
        )?;
        Ok(())
    }

    fn select_station_check_steps(&self)
    -> Result<Vec<StationCheckStepItem>,Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let list = conn.query_map("SELECT Id,StationUuid,CheckUuid,Url,UrlType,Error,StepUuid,ParentStepUuid,InsertTime FROM StationCheckStep",
            |(id,stationuuid,checkuuid,url,urltype,error,stepuuid,parent_stepuuid,inserttime)| {
                let inserttime = chrono::DateTime::<chrono::Utc>::from_utc(inserttime, chrono::Utc);
            StationCheckStepItem{
                id,stepuuid,parent_stepuuid,checkuuid,stationuuid,url,urltype,error,inserttime
            }
        })?;
        Ok(list)
    }

    fn select_station_check_steps_by_stations(&self, stationuuids: &[String])
    -> Result<Vec<StationCheckStepItem>,Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        if stationuuids.len() > 0{
            let mut select_params: Vec<Value> = vec![];
            let mut select_query = vec![];
            for stationuuid in stationuuids {
                select_params.push(stationuuid.into());
                select_query.push("?");
            }

            let query = format!("SELECT Id,StationUuid,CheckUuid,Url,UrlType,Error,StepUuid,ParentStepUuid,InsertTime FROM StationCheckStep WHERE StationUuid IN ({})", select_query.join(","));
            let list = conn.exec_map(query, select_params,
                |(id,stationuuid,checkuuid,url,urltype,error,stepuuid,parent_stepuuid,inserttime)| {
                let inserttime = chrono::DateTime::<chrono::Utc>::from_utc(inserttime, chrono::Utc);
                StationCheckStepItem{
                    id,stepuuid,parent_stepuuid,checkuuid,stationuuid,url,urltype,error,inserttime
                }
            })?;
            Ok(list)
        } else {
            Ok(vec![])
        }
    }

    fn delete_old_station_check_steps(&mut self, seconds: u32)
    -> Result<(),Box<dyn std::error::Error>> {
        let delete_never_working_query = "DELETE FROM StationCheckStep WHERE InsertTime < UTC_TIMESTAMP() - INTERVAL :seconds SECOND";
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(delete_never_working_query, params!(seconds))?;
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

fn fix_url(u: &str, allow_empty: bool) -> Result<String, Box<dyn std::error::Error>> {
    let url_str = u.trim();
    if url_str.is_empty() {
        if allow_empty {
            return Ok(url_str.to_string());
        }else{
            return Err(Box::new(DbError::AddStationError(String::from("empty url not allowed"))));
        }
    }
    let url = Url::parse(url_str)?;
    let scheme = url.scheme().to_lowercase();
    if !scheme.eq("http") && !scheme.eq("https") {
        return Err(Box::new(DbError::AddStationError(String::from("unknown url scheme"))));
    }
    let url = url.to_string();
    Ok(url)
}

fn ok_output<U,V>(t: Result<U, V>, msg: &str) -> Option<U> where V: std::error::Error {
    match t {
        Ok(u) => Some(u),
        Err(v) => {error!("{}: {}", msg, v); None}
    }
}