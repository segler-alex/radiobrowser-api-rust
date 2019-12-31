use crate::db::models::StationCheckItemNew;
use std::thread;
use crate::time;
use crate::api::db;
use crate::api::api_error;
use crate::api::data::StationHistoryCurrent;
use crate::api::data::StationHistoryV0;
use crate::api::data::StationCheck;
use crate::api::data::StationCheckV0;
use crate::api::data::Status;
use crate::db::DbConnection;

pub fn run<A: 'static>(connection: db::Connection, connection_new: A, mirrors: Vec<String>, pull_interval: u64) where A: DbConnection, A: std::marker::Send {
    thread::spawn(move || {
        loop {
            for server in mirrors.iter() {
                let result = pull_server(&connection, &connection_new, &server);
                match result {
                    Ok(_) => {
                    },
                    Err(err) => {
                        error!("Error pulling from '{}': {}", server, err);
                    }
                }
            }
            thread::sleep(time::Duration::from_secs(pull_interval));
        }
    });
}

fn get_remote_version(server: &str) -> Result<u32,Box<dyn std::error::Error>> {
    debug!("Check server status of '{}' ..", server);
    let path = format!("{}/json/stats",server);
    let status: Status = reqwest::get(&path)?.json()?;
    Ok(status.supported_version)
}

fn pull_history(server: &str, api_version: u32, lastid: Option<String>) -> Result<Vec<StationHistoryCurrent>, Box<dyn std::error::Error>> {
    trace!("Pull history from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/stations/changed?lastchangeuuid={}",server, id),
        None => format!("{}/json/stations/changed",server),
    };
    trace!("{}", path);
    let mut result = reqwest::get(&path)?;
    match api_version {
        0 => {
            let list: Vec<StationHistoryV0> = result.json()?;
            let list_current: Vec<StationHistoryCurrent> = list.iter().map(|x| x.into()).collect();
            Ok(list_current)
        },
        1 => {
            let list: Vec<StationHistoryCurrent> = result.json()?;
            Ok(list)
        },
        _ => {
            Err(Box::new(api_error::ApiError::UnknownApiVersion(api_version)))
        }
    }
}

fn pull_checks(server: &str, api_version: u32, lastid: Option<String>) -> Result<Vec<StationCheck>, Box<dyn std::error::Error>> {
    trace!("Pull checks from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/checks?lastcheckuuid={}",server, id),
        None => format!("{}/json/checks",server),
    };
    trace!("{}", path);
    let mut result = reqwest::get(&path)?;
    match api_version {
        0 => {
            let list: Vec<StationCheckV0> = result.json()?;
            let list_current: Vec<StationCheck> = list.iter().map(|x| x.into()).collect();
            Ok(list_current)
        },
        1 => {
            let list: Vec<StationCheck> = result.json()?;
            Ok(list)
        },
        _ => {
            Err(Box::new(api_error::ApiError::UnknownApiVersion(api_version)))
        }
    }
}

fn pull_server<A>(connection: &db::Connection, connection_new: &A, server: &str) -> Result<(),Box<dyn std::error::Error>> where A: DbConnection {
    let chunksize = 1000;

    let api_version = get_remote_version(server)?;
    let lastid = connection_new.get_pull_server_lastid(server);
    let list = pull_history(server, api_version, lastid)?;
    let len = list.len();

    trace!("Incremental station change sync ({})..", list.len());
    let mut station_change_count = 0;
    for station in list {
        let changeuuid = station.changeuuid.clone();
        connection.insert_station_by_change(station)?;
        station_change_count = station_change_count + 1;

        if station_change_count % chunksize == 0 || station_change_count == len {
            trace!("Insert {} station changes..", chunksize);
            connection_new.set_pull_server_lastid(server, &changeuuid)?;
            trace!("..done");
        }
    }

    let lastcheckid = connection_new.get_pull_server_lastcheckid(server);
    let list_checks = pull_checks(server, api_version, lastcheckid)?;
    let len = list_checks.len();

    trace!("Incremental checks sync ({})..", list_checks.len());
    let mut station_check_count = 0;
    let mut list_checks_converted = vec![];
    for check in list_checks {
        let changeuuid = check.checkuuid.clone();
        let value: StationCheckItemNew = check.into();
        list_checks_converted.push(value);
        station_check_count = station_check_count + 1;

        if station_check_count % chunksize == 0 || station_check_count == len {
            trace!("Insert {} checks..", chunksize);
            connection_new.insert_checks(&list_checks_converted)?;
            connection_new.update_station_with_check_data(&list_checks_converted)?;
            connection_new.set_pull_server_lastcheckid(server, &changeuuid)?;
            trace!("..done");            
        }
    }

    info!("Pull from '{}' OK (Added station changes: {}, Added station checks: {})", server, station_change_count, station_check_count);
    Ok(())
}

impl From<StationCheck> for StationCheckItemNew {
    fn from(item: StationCheck) -> Self {
        StationCheckItemNew {
            station_uuid: item.stationuuid,
            check_ok: item.ok == 1,
            bitrate: item.bitrate,
            codec: item.codec,
            hls: item.hls == 1,
            source: item.source,
            url: item.urlcache,
        }
    }
}