use std::error::Error;
use mysql;
use check::models::StationItem;
use check::models::StationCheckItemNew;

pub fn get_stations_to_check(pool: &mysql::Pool, hours: u32, itemcount: u32) -> Vec<StationItem> {
    let query = format!("SELECT StationID,StationUuid,Name,Codec,Bitrate,Hls,LastCheckOk,UrlCache,Url,Favicon,Homepage FROM Station WHERE LastCheckTime IS NULL OR LastCheckTime < NOW() - INTERVAL {} HOUR ORDER BY RAND() LIMIT {}", hours, itemcount);
    get_stations_query(pool, query)
}

fn get_stations_query(pool: &mysql::Pool, query: String) -> Vec<StationItem> {
    let mut stations: Vec<StationItem> = vec![];
    let results = pool.prep_exec(query, ());
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

pub fn get_station_count_broken(pool: &mysql::Pool) -> u32 {
    let query = format!("SELECT COUNT(*) AS Items FROM radio.Station WHERE LastCheckOK=0 OR LastCheckOK IS NULL");
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let items: u32 = row.take_opt("Items").unwrap_or(Ok(0)).unwrap_or(0);
            return items;
        }
    }
    return 0;
}

pub fn get_station_count_working(pool: &mysql::Pool) -> u32 {
    let query = format!("SELECT COUNT(*) AS Items FROM radio.Station WHERE LastCheckOK=1");
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let items: u32 = row.take_opt("Items").unwrap_or(Ok(0)).unwrap_or(0);
            return items;
        }
    }
    return 0;
}

pub fn get_station_count_todo(pool: &mysql::Pool, hours: i32) -> u32 {
    let query = format!("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckTime IS NULL OR LastCheckTime < NOW() - INTERVAL {} HOUR", hours);
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let items: u32 = row.take_opt("Items").unwrap_or(Ok(0)).unwrap_or(0);
            return items;
        }
    }
    return 0;
}

pub fn get_checks(pool: &mysql::Pool, hours: u32, source: &str) -> u32 {
    let query = format!("SELECT COUNT(*) AS Items FROM StationCheckHistory WHERE Source=? AND CheckTime > NOW() - INTERVAL {} HOUR", hours);
    let results = pool.prep_exec(query, (source,));
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let items: u32 = row.take_opt("Items").unwrap_or(Ok(0)).unwrap_or(0);
            return items;
        }
    }
    return 0;
}

pub fn get_deletable_never_working(pool: &mysql::Pool, hours: u32) -> u32 {
    let query = format!("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOkTime IS NULL AND Creation < NOW() - INTERVAL {} HOUR", hours);
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let items: u32 = row.take_opt("Items").unwrap_or(Ok(0)).unwrap_or(0);
            return items;
        }
    }
    return 0;
}

pub fn get_deletable_were_working(pool: &mysql::Pool, hours: u32) -> u32 {
    let query = format!("SELECT COUNT(*) AS Items FROM Station WHERE LastCheckOk=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < NOW() - INTERVAL {} HOUR", hours);
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let items: u32 = row.take_opt("Items").unwrap_or(Ok(0)).unwrap_or(0);
            return items;
        }
    }
    return 0;
}

pub fn delete_never_working(pool: &mysql::Pool, hours: u32) {
    let query = format!("DELETE FROM Station WHERE LastCheckOkTime IS NULL AND Creation < NOW() - INTERVAL {} HOUR", hours);
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(());
    match result {
        Ok(_) => {},
        Err(err) => {debug!("{}",err);}
    }
}

pub fn delete_were_working(pool: &mysql::Pool, hours: u32) {
    let query = format!("DELETE FROM Station WHERE LastCheckOk=0 AND LastCheckOkTime IS NOT NULL AND LastCheckOkTime < NOW() - INTERVAL {} HOUR", hours);
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(());
    match result {
        Ok(_) => {},
        Err(err) => {debug!("{}",err);}
    }
}

pub fn insert_check(pool: &mysql::Pool,item: &StationCheckItemNew) -> Result<(), Box<std::error::Error>> {
    let query = "DELETE FROM StationCheck WHERE StationUuid=:stationuuid AND Source=:source";
    let mut my_stmt = pool.prepare(query)?;
    my_stmt.execute(params!(
        "stationuuid" => &item.station_uuid,
        "source" => &item.source
    ))?;

    let query2 = "INSERT INTO StationCheck(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) VALUES(?,UUID(),?,?,?,?,?,NOW(),?)";
    let mut my_stmt2 = pool.prepare(query2)?;
    my_stmt2.execute((&item.station_uuid,&item.source,&item.codec,&item.bitrate,&item.hls,&item.check_ok,&item.url))?;

    let query3 = "INSERT INTO StationCheckHistory(StationUuid,CheckUuid,Source,Codec,Bitrate,Hls,CheckOK,CheckTime,UrlCache) VALUES(?,UUID(),?,?,?,?,?,NOW(),?)";
    let mut my_stmt3 = pool.prepare(query3)?;
    my_stmt3.execute((&item.station_uuid,&item.source,&item.codec,&item.bitrate,&item.hls,&item.check_ok,&item.url))?;
    Ok(())
}

pub fn update_station(pool: &mysql::Pool,item: &StationCheckItemNew){
    let mut query: String = String::from("UPDATE Station SET LastCheckTime=NOW(),LastCheckOkTime=NOW(),LastCheckOk=?,Codec=?,Bitrate=?,UrlCache=? WHERE StationUuid=?");
    if !item.check_ok{
        query = format!("UPDATE Station SET LastCheckTime=NOW(),LastCheckOk=?,Codec=?,Bitrate=?,UrlCache=? WHERE StationUuid=?");
    }
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute((&item.check_ok,&item.codec,&item.bitrate,&item.url,&item.station_uuid));
    match result {
        Ok(_) => {},
        Err(err) => {debug!("{}",err);}
    }
}

pub fn delete_old_checks(pool: &mysql::Pool, hours: u32) {
    let query = format!("DELETE FROM StationCheckHistory WHERE CheckTime < NOW() - INTERVAL {} HOUR", hours);
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(());
    match result {
        Ok(_) => {},
        Err(err) => {debug!("{}",err);}
    }

    let query = format!("DELETE FROM StationCheck WHERE CheckTime < NOW() - INTERVAL {} HOUR", hours);
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(());
    match result {
        Ok(_) => {},
        Err(err) => {debug!("{}",err);}
    }
}

pub fn delete_old_clicks(pool: &mysql::Pool, hours: u32) {
    let query = format!("DELETE FROM StationClick WHERE ClickTimestamp < NOW() - INTERVAL {} HOUR", hours);
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute(());
    match result {
        Ok(_) => {},
        Err(err) => {debug!("{}",err);}
    }
}

pub fn new(connection_str: &str) -> Result<mysql::Pool, Box<Error>> {
    let pool = mysql::Pool::new(connection_str)?;
    Ok(pool)
}
