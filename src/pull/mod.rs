mod api_error;

use std::error::Error;
use std::thread;
use crate::time;
use std::convert::TryFrom;

use crate::api::data::StationHistoryCurrent;
use crate::api::data::StationHistoryV0;
use crate::api::data::StationCheck;
use crate::api::data::StationCheckV0;
use crate::api::data::Status;
use crate::db::DbConnection;
use crate::db::connect;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationChangeItemNew;

fn pull_worker(connection_string: String, mirrors: &Vec<String>) -> Result<(),Box<dyn Error>> {
    let pool = connect(connection_string)?;
    for server in mirrors.iter() {
        let result = pull_server(&pool, &server);
        match result {
            Ok(_) => {
            },
            Err(err) => {
                error!("Error pulling from '{}': {}", server, err);
            }
        }
    }
    Ok(())
}

pub fn start(connection_string: String, mirrors: Vec<String>, pull_interval: u64) {
    if mirrors.len() > 0 {
        thread::spawn(move || {
            loop {
                let result = pull_worker(connection_string.clone(), &mirrors);
                match result {
                    Ok(_) => {
                    },
                    Err(err) => {
                        error!("Error in pull worker: {}", err);
                    }
                }
                thread::sleep(time::Duration::from_secs(pull_interval));
            }
        });
    }
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
            let mut list: Vec<StationCheckV0> = result.json()?;
            let list_current: Vec<StationCheck> = list.drain(..).filter_map(|x| StationCheck::try_from(x).ok()).collect();
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

fn pull_server(connection_new: &Box<dyn DbConnection>, server: &str) -> Result<(),Box<dyn std::error::Error>> {
    let chunksize = 1000;

    let api_version = get_remote_version(server)?;
    let lastid = connection_new.get_pull_server_lastid(server);
    let list = pull_history(server, api_version, lastid)?;
    let len = list.len();

    trace!("Incremental station change sync ({})..", list.len());
    let mut station_change_count = 0;
    let mut list_stations: Vec<StationChangeItemNew> = vec![];
    for station in list {
        let changeuuid = station.changeuuid.clone();
        station_change_count = station_change_count + 1;
        list_stations.push(station.into());

        if station_change_count % chunksize == 0 || station_change_count == len {
            trace!("Insert {} station changes..", list_stations.len());
            connection_new.insert_station_by_change(&list_stations)?;
            connection_new.set_pull_server_lastid(server, &changeuuid)?;
            list_stations.clear();
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
            trace!("Insert {} checks..", list_checks_converted.len());
            connection_new.insert_checks(&list_checks_converted)?;
            connection_new.update_station_with_check_data(&list_checks_converted, false)?;
            connection_new.set_pull_server_lastcheckid(server, &changeuuid)?;
            list_checks_converted.clear();
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

            metainfo_overrides_database: item.metainfo_overrides_database.unwrap_or_default() == 1,
            public: item.public.map(|x| x == 1),
            name: item.name,
            description: item.description,
            tags: item.tags,
            countrycode: item.countrycode,
            homepage: item.homepage,
            favicon: item.favicon,
            loadbalancer: item.loadbalancer,
        }
    }
}

impl From<StationHistoryCurrent> for StationChangeItemNew {
    fn from(item: StationHistoryCurrent) -> Self {
        StationChangeItemNew {
            name: item.name,
            url: item.url,
            homepage: item.homepage,
            favicon: item.favicon,
            country: item.country,
            state: item.state,
            countrycode: item.countrycode,
            language: item.language,
            tags: item.tags,
            votes: item.votes,
        
            changeuuid: item.changeuuid,
            stationuuid: item.stationuuid,
        }
    }
}