use std::thread;
use time;
use api;
use api::db;
use api::api_error;
use api::data::StationHistoryCurrent;
use api::data::StationHistoryV0;

pub fn run(connection: db::Connection, mirrors: Vec<String>, pull_interval: u64){
    thread::spawn(move || {
        loop {
            for server in mirrors.iter() {
                let result = pull_server(&connection, &server);
                match result {
                    Ok(_) => {
                    },
                    Err(err) => {
                        println!("Error pulling from '{}': {}", server, err);
                    }
                }
            }
            thread::sleep(time::Duration::from_secs(pull_interval));
        }
    });
}

fn get_remote_version(server: &str) -> Result<u32,Box<std::error::Error>> {
    println!("Check server status of '{}' ..", server);
    let path = format!("{}/json/stats",server);
    let status: api::Status = reqwest::get(&path)?.json()?;
    Ok(status.supported_version)
}

fn pull_history(server: &str, api_version: u32, lastid: Option<String>) -> Result<Vec<StationHistoryCurrent>, Box<std::error::Error>> {
    println!("Pull from '{}' (API: {}) ..", server, api_version);
    let path = match lastid {
        Some(id) => format!("{}/json/stations/changed?lastchangeuuid={}",server, id),
        None => format!("{}/json/stations/changed",server),
    };
    println!("{}", path);
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

fn pull_server(connection: &db::Connection, server: &str) -> Result<(),Box<std::error::Error>> {
    let api_version = get_remote_version(server)?;
    let lastid = connection.get_pull_server_lastid(server);
    let list = pull_history(server, api_version, lastid)?;

    /*if connection.is_empty()? {
        println!("Initial sync ({})..", list.len());
        let chunksize = 10;
        connection.insert_station_changes(&list[0..chunksize])?;
    }else{*/
        print!("Incremental sync ({})..", list.len());
        let mut i = 0;
        for station in list {
            let changeuuid = station.changeuuid.clone();
            connection.insert_station_by_change(station)?;
            i = i + 1;

            if i % 100 == 0 {
                print!(".");
                connection.set_pull_server_lastid(server, &changeuuid)?;
            }
        }
        println!("");
    //}
    println!("Pull from '{}' OK", server);
    Ok(())
}