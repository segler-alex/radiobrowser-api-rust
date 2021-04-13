mod pull_error;

use std::error::Error;
use std::thread;
use crate::time;
use std::convert::TryFrom;

use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;
use reqwest::header::USER_AGENT;

use crate::api::data::StationHistoryCurrent;
use crate::api::data::StationHistoryV0;
use crate::api::data::StationCheck;
use crate::api::data::StationCheckV0;
use crate::api::data::Status;
use crate::api::data::StationClick;
use crate::api::data::StationClickV0;
use crate::api::data::Station;
use crate::api::data::StationV0;
use crate::db::DbConnection;
use crate::db::connect;
use crate::db::models::StationCheckItemNew;
use crate::db::models::StationChangeItemNew;
use crate::db::models::StationClickItemNew;

fn add_default_request_headers(req: RequestBuilder) -> RequestBuilder {
    let pkg_version = env!("CARGO_PKG_VERSION");
    req.header(USER_AGENT, format!("radiobrowser-api-rust/{}",pkg_version))
}

fn pull_worker(client: &Client, connection_string: String, mirrors: &Vec<String>, chunk_size_changes: usize, chunk_size_checks: usize) -> Result<(),Box<dyn Error>> {
    let pool = connect(connection_string)?;
    for server in mirrors.iter() {
        let result = pull_server(client, &pool, &server, chunk_size_changes, chunk_size_checks);
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

pub fn start(connection_string: String, mirrors: Vec<String>, pull_interval: u64, chunk_size_changes: usize, chunk_size_checks: usize) {
    if mirrors.len() > 0 {
        thread::spawn(move || {
            let client = Client::new();
            loop {
                let result = pull_worker(&client, connection_string.clone(), &mirrors, chunk_size_changes, chunk_size_checks);
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

fn get_remote_version(client: &Client, server: &str) -> Result<u32,Box<dyn std::error::Error>> {
    debug!("Check server status of '{}' ..", server);
    let path = format!("{}/json/stats",server);
    let status: Status = add_default_request_headers(client.get(&path)).send()?.json()?;
    Ok(status.supported_version)
}

fn pull_history(client: &Client, server: &str, api_version: u32, lastid: Option<String>, chunk_size_changes: usize) -> Result<Vec<StationHistoryCurrent>, Box<dyn std::error::Error>> {
    trace!("Pull history from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/stations/changed?lastchangeuuid={}&limit={}",server, id, chunk_size_changes),
        None => format!("{}/json/stations/changed?limit={}",server,chunk_size_changes),
    };
    trace!("{}", path);
    let result = add_default_request_headers(client.get(&path)).send()?;
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
            Err(Box::new(pull_error::PullError::UnknownApiVersion(api_version)))
        }
    }
}

fn pull_checks(client: &Client, server: &str, api_version: u32, lastid: Option<String>, chunk_size_checks: usize) -> Result<Vec<StationCheck>, Box<dyn std::error::Error>> {
    trace!("Pull checks from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/checks?lastcheckuuid={}&limit={}",server, id, chunk_size_checks),
        None => format!("{}/json/checks?limit={}",server,chunk_size_checks),
    };
    trace!("{}", path);
    let result = add_default_request_headers(client.get(&path)).send()?;
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
            Err(Box::new(pull_error::PullError::UnknownApiVersion(api_version)))
        }
    }
}

/// Pull all known changes for a single station from remote
fn pull_stations_history(client: &Client, server: &str, api_version: u32, stationuuid: &str) -> Result<Vec<StationHistoryCurrent>, Box<dyn std::error::Error>>{
    trace!("Pull station history from '{}' for station '{}' (API: {}) ..", server, stationuuid, api_version);
    let path = format!("{}/json/stations/changed/{}", server, stationuuid);
    trace!("{}", path);
    let result = add_default_request_headers(client.get(&path)).send()?;
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
            Err(Box::new(pull_error::PullError::UnknownApiVersion(api_version)))
        }
    }
}

fn pull_clicks(client: &Client, server: &str, api_version: u32, lastid: Option<String>) -> Result<Vec<StationClick>, Box<dyn std::error::Error>> {
    trace!("Pull clicks from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/clicks?lastclickuuid={}",server, id),
        None => format!("{}/json/clicks",server),
    };
    trace!("{}", path);
    let result = add_default_request_headers(client.get(&path)).send()?;
    match api_version {
        0 => {
            let mut list: Vec<StationClickV0> = result.json()?;
            let list_current: Vec<StationClick> = list.drain(..).filter_map(|x| StationClick::try_from(x).ok()).collect();
            Ok(list_current)
        },
        1 => {
            let list: Vec<StationClick> = result.json()?;
            Ok(list)
        },
        _ => {
            Err(Box::new(pull_error::PullError::UnknownApiVersion(api_version)))
        }
    }
}

fn pull_stations(client: &Client, server: &str, api_version: u32) -> Result<Vec<Station>, Box<dyn Error>> {
    let path = format!("{}/json/stations",server);
    trace!("{}", path);
    let result = add_default_request_headers(client.get(&path)).send()?;
    match api_version {
        0 => {
            let mut list: Vec<StationV0> = result.json()?;
            let list_current: Vec<Station> = list.drain(..).map(|x| x.into()).collect();
            Ok(list_current)
        },
        1 => {
            let list: Vec<Station> = result.json()?;
            Ok(list)
        },
        _ => {
            Err(Box::new(pull_error::PullError::UnknownApiVersion(api_version)))
        }
    }
}

fn pull_server(client: &Client, connection_new: &Box<dyn DbConnection>, server: &str, chunk_size_changes: usize, chunk_size_checks: usize) -> Result<(),Box<dyn std::error::Error>> {
    let insert_chunksize = 2000;
    let mut station_change_count = 0;
    let mut station_check_count = 0;
    let mut station_click_count = 0;
    let mut station_missing_count = 0;

    let api_version = get_remote_version(client, server)?;
    loop {
        let lastid = connection_new.get_pull_server_lastid(server)?;
        let mut list_changes = pull_history(client, server, api_version, lastid, chunk_size_changes)?;
        let len = list_changes.len();

        trace!("Incremental station change sync ({})..", len);
        let list_stations: Vec<StationChangeItemNew> = list_changes.drain(..).map(|item| item.into()).collect();
        for chunk in list_stations.chunks(insert_chunksize) {
            station_change_count = station_change_count + chunk.len();
            let last = chunk.last();

            if let Some(last) = last {
                trace!("Insert {} station changes..", chunk.len());
                connection_new.insert_station_by_change(chunk)?;
                connection_new.set_pull_server_lastid(server, &last.changeuuid)?;
            }
        }

        if len < chunk_size_changes {
            break;
        }
    }

    loop {
        let lastcheckid = connection_new.get_pull_server_lastcheckid(server)?;
        let mut list_checks = pull_checks(client, server, api_version, lastcheckid, chunk_size_checks)?;
        let len = list_checks.len();

        trace!("Incremental checks sync ({})..", len);
        let list_checks_converted: Vec<StationCheckItemNew> = list_checks.drain(..).map(|item| item.into()).collect();
        station_check_count = station_check_count + list_checks_converted.len();

        for chunk in list_checks_converted.chunks(insert_chunksize) {
            trace!("Insert {} checks..", chunk.len());
            let (_ignored_uuids_check_existing, checks_ignored_station_missing, inserted) = connection_new.insert_checks(chunk.to_vec())?;
            trace!("Inserted checks ({})..", inserted.len());

            let last = chunk.last();
            connection_new.update_station_with_check_data(&inserted, false)?;
            if let Some(last) = last {
                let checkuuid = &last.checkuuid;
                if let Some(checkuuid) = checkuuid {
                    connection_new.set_pull_server_lastcheckid(server, &checkuuid)?;
                }
            }

            // try to fetch stations from remote if they are not in local database
            if checks_ignored_station_missing.len() > 0 {
                warn!("Pulling stations for {} checks missing from local database", checks_ignored_station_missing.len());
                station_missing_count += checks_ignored_station_missing.len();
                let mut station_uuids_to_pull: Vec<&String> = checks_ignored_station_missing
                    .iter()
                    .filter(|check| {
                        if !check.check_ok {
                            trace!("Filtered check for broken station {}", check.station_uuid);
                        }
                        check.check_ok
                    })
                    .map(|check| &check.station_uuid)
                    .collect();
                station_uuids_to_pull.sort();
                let uuids_before_dedup = station_uuids_to_pull.len();
                station_uuids_to_pull.dedup();
                let uuids_after_dedup = station_uuids_to_pull.len();
                trace!("Dedup pulled station list removed {} stations", uuids_before_dedup - uuids_after_dedup);
                trace!("Pulling {} stations missing from local database", station_uuids_to_pull.len());
                let changes: Vec<StationChangeItemNew> = station_uuids_to_pull
                    .drain(..)
                    .map(|station_uuid| {
                        let result = pull_stations_history(client, server, api_version, station_uuid);
                        match result {
                            Ok(r) => Ok(r),
                            Err(e) => {
                                error!("Error on pulling history for station '{}': {}", station_uuid, e);
                                Err(e)
                            }
                        }
                    })
                    .filter(|items| items.is_ok())
                    .flat_map(|items| items.unwrap())
                    .map(|change| change.into())
                    .collect();
                
                trace!("Inserting missing stations ({})..", changes.len());
                if changes.len() > 0 {
                    connection_new.insert_station_by_change(&changes)?;
                }
                trace!("Insert checks ({})..", checks_ignored_station_missing.len());
                let (_ignored_uuids_check_existing, _ignored_uuids_station_missing, inserted) = connection_new.insert_checks(checks_ignored_station_missing)?;
                trace!("Inserted checks ({})..", inserted.len());
            }
        }
        if len < chunk_size_checks {
            break;
        }
    }

    loop {
        // default chunksize from server is 10000
        let download_chunksize = 10000;
        let insert_chunksize = 5000;
        let lastclickuuid = connection_new.get_pull_server_lastclickid(server)?;
        let list_clicks = pull_clicks(client, server, api_version, lastclickuuid)?;
        let len = list_clicks.len();
        let mut local_station_click_count = 0;

        trace!("Incremental clicks sync({})..", list_clicks.len());
        let mut list_clicks_converted = vec![];
        for click in list_clicks {
            let clickuuid = click.clickuuid.clone();
            let value: StationClickItemNew = click.into();
            list_clicks_converted.push(value);
            station_click_count = station_click_count + 1;
            local_station_click_count = local_station_click_count + 1;

            if station_click_count % insert_chunksize == 0 || local_station_click_count == len {
                trace!("Insert {} clicks..", list_clicks_converted.len());
                connection_new.insert_clicks(&list_clicks_converted)?;
                connection_new.set_pull_server_lastclickid(server, &clickuuid)?;
                list_clicks_converted.clear();
            }
        }

        if len < download_chunksize {
            // last chunk reached
            break;
        }
    }
    connection_new.update_stations_clickcount()?;

    {
        // this section is bad, because it
        // makes the incremental import before meaningless :(
        // but we need it to keep votes in sync
        // we could make votes more like clicks, then we could also
        // make votes incremental
        let list_stations = pull_stations(client, server, api_version)?;
        connection_new.sync_votes(list_stations)?;
    }

    debug!("Pull from '{}' OK (Added station changes: {}, Added station checks: {}, Added station clicks: {}, Added missing stations: {})", server, station_change_count, station_check_count, station_click_count, station_missing_count);
    Ok(())
}

impl From<StationCheck> for StationCheckItemNew {
    fn from(item: StationCheck) -> Self {
        StationCheckItemNew {
            checkuuid: Some(item.checkuuid),
            station_uuid: item.stationuuid,
            check_ok: item.ok == 1,
            bitrate: item.bitrate,
            sampling: item.sampling,
            codec: item.codec,
            hls: item.hls == 1,
            source: item.source,
            url: item.urlcache,
            timestamp: Some(item.timestamp),

            metainfo_overrides_database: item.metainfo_overrides_database.unwrap_or_default() == 1,
            public: item.public.map(|x| x == 1),
            name: item.name,
            description: item.description,
            tags: item.tags,
            countrycode: item.countrycode,
            countrysubdivisioncode: item.countrysubdivisioncode,
            languagecodes: item.languagecodes,
            homepage: item.homepage,
            favicon: item.favicon,
            loadbalancer: item.loadbalancer,
            do_not_index: item.do_not_index.map(|x| x == 1),
            timing_ms: item.timing_ms.unwrap_or(0),
            server_software: item.server_software,
            ssl_error: item.ssl_error.unwrap_or(0) == 1,
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

impl From<StationClick> for StationClickItemNew {
    fn from(item: StationClick) -> Self {
        StationClickItemNew {
            clickuuid: item.clickuuid,
            stationuuid: item.stationuuid,
            clicktimestamp: item.clicktimestamp,
            ip: String::from(""),
        }
    }
}