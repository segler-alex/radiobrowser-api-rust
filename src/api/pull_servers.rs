use std::thread;
use time;
use api::db;
use api::api_error;
use api::data::StationHistoryCurrent;
use api::data::StationHistoryV0;
use api::data::StationCheck;
use api::data::StationCheckV0;
use api::data::Status;

pub fn run(connection: db::Connection, mirrors: Vec<String>, pull_interval: u64){
    thread::spawn(move || {
        loop {
            for server in mirrors.iter() {
                let result = pull_server(&connection, &server);
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

fn pull_server(connection: &db::Connection, server: &str) -> Result<(),Box<dyn std::error::Error>> {
    let api_version = get_remote_version(server)?;
    let lastid = connection.get_pull_server_lastid(server);
    let list = pull_history(server, api_version, lastid)?;
    let len = list.len();

    /*if connection.is_empty()? {
        println!("Initial sync ({})..", list.len());
        let chunksize = 10;
        connection.insert_station_changes(&list[0..chunksize])?;
    }else{*/
    
    trace!("Incremental station change sync ({})..", list.len());
    let mut station_change_count = 0;
    for station in list {
        let changeuuid = station.changeuuid.clone();
        connection.insert_station_by_change(station)?;
        station_change_count = station_change_count + 1;

        if station_change_count % 100 == 0 {
            connection.set_pull_server_lastid(server, &changeuuid)?;
        }
        if station_change_count == len {
            connection.set_pull_server_lastid(server, &changeuuid)?;
        }
    }
    //}

    let lastcheckid = connection.get_pull_server_lastcheckid(server);
    let list_checks = pull_checks(server, api_version, lastcheckid)?;
    let len = list_checks.len();

    trace!("Incremental checks sync ({})..", list_checks.len());
    let mut station_check_count = 0;
    for check in list_checks {
        let changeuuid = check.checkuuid.clone();
        connection.update_station_with_check_data(&check)?;
        connection.insert_pulled_station_check(check)?;
        station_check_count = station_check_count + 1;

        if station_check_count % 100 == 0 {
            connection.set_pull_server_lastcheckid(server, &changeuuid)?;
        }
        if station_check_count == len {
            connection.set_pull_server_lastcheckid(server, &changeuuid)?;
        }
    }

    info!("Pull from '{}' OK (Added station changes: {}, Added station checks: {})", server, station_change_count, station_check_count);
    Ok(())
}