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

fn pull_worker(client: &Client, connection_string: String, mirrors: &Vec<String>) -> Result<(),Box<dyn Error>> {
    let pool = connect(connection_string)?;
    for server in mirrors.iter() {
        let result = pull_server(client, &pool, &server);
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
            let client = Client::new();
            loop {
                let result = pull_worker(&client, connection_string.clone(), &mirrors);
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

fn pull_history(client: &Client, server: &str, api_version: u32, lastid: Option<String>) -> Result<Vec<StationHistoryCurrent>, Box<dyn std::error::Error>> {
    trace!("Pull history from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/stations/changed?lastchangeuuid={}",server, id),
        None => format!("{}/json/stations/changed",server),
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

fn pull_checks(client: &Client, server: &str, api_version: u32, lastid: Option<String>) -> Result<Vec<StationCheck>, Box<dyn std::error::Error>> {
    trace!("Pull checks from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/checks?lastcheckuuid={}",server, id),
        None => format!("{}/json/checks",server),
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

fn pull_server(client: &Client, connection_new: &Box<dyn DbConnection>, server: &str) -> Result<(),Box<dyn std::error::Error>> {
    let insert_chunksize = 1000;
    let mut station_change_count = 0;
    let mut station_check_count = 0;
    let mut station_click_count = 0;

    let api_version = get_remote_version(client, server)?;
    {
        let lastid = connection_new.get_pull_server_lastid(server)?;
        let list_changes = pull_history(client, server, api_version, lastid)?;
        let len = list_changes.len();

        trace!("Incremental station change sync ({})..", len);
        let mut list_stations: Vec<StationChangeItemNew> = vec![];
        for station in list_changes {
            let changeuuid = station.changeuuid.clone();
            station_change_count = station_change_count + 1;
            list_stations.push(station.into());

            if station_change_count % insert_chunksize == 0 || station_change_count == len {
                trace!("Insert {} station changes..", list_stations.len());
                connection_new.insert_station_by_change(&list_stations)?;
                connection_new.set_pull_server_lastid(server, &changeuuid)?;
                list_stations.clear();
            }
        }
    }

    {
        let lastcheckid = connection_new.get_pull_server_lastcheckid(server)?;
        let list_checks = pull_checks(client, server, api_version, lastcheckid)?;
        let len = list_checks.len();

        trace!("Incremental checks sync ({})..", len);
        let mut list_checks_converted = vec![];
        for check in list_checks {
            let checkuuid = check.checkuuid.clone();
            let value: StationCheckItemNew = check.into();
            list_checks_converted.push(value);
            station_check_count = station_check_count + 1;

            if station_check_count % insert_chunksize == 0 || station_check_count == len {
                trace!("Insert {} checks..", list_checks_converted.len());
                let ignored_uuids = connection_new.insert_checks(&list_checks_converted)?;
                connection_new.update_station_with_check_data(&list_checks_converted.drain(..).filter(|item| match &item.checkuuid { Some(checkuuid) => !ignored_uuids.contains(checkuuid), None => true }).collect(), false)?;
                connection_new.set_pull_server_lastcheckid(server, &checkuuid)?;
                list_checks_converted.clear();
            }
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
        let list_stations = pull_stations(client, server, api_version)?;
        connection_new.sync_votes(list_stations)?;
    }

    debug!("Pull from '{}' OK (Added station changes: {}, Added station checks: {}, Added station clicks: {})", server, station_change_count, station_check_count, station_click_count);
    Ok(())
}

impl From<StationCheck> for StationCheckItemNew {
    fn from(item: StationCheck) -> Self {
        StationCheckItemNew {
            checkuuid: Some(item.checkuuid),
            station_uuid: item.stationuuid,
            check_ok: item.ok == 1,
            bitrate: item.bitrate,
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

impl From<StationClick> for StationClickItemNew {
    fn from(item: StationClick) -> Self {
        StationClickItemNew {
            clickuuid: item.clickuuid,
            stationuuid: item.stationuuid,
            clicktimestamp: item.clicktimestamp,
            ip: String::from(""),
            stationid: 0,
        }
    }
}