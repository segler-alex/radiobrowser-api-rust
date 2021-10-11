#[macro_use]
extern crate clap;
#[macro_use]
extern crate mysql;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

use crate::pull::UuidWithTime;
use core::fmt::Display;
use core::fmt::Formatter;
use reqwest::blocking::Client;
use std::error::Error;
use std::time::Duration;
use std::time::Instant;
use std::{thread, time};

mod api;
mod check;
mod checkserver;
mod cleanup;
mod config;
mod db;
mod logger;
mod pull;
mod refresh;

#[derive(Debug, Clone)]
enum MainError {
    ConfigLoadError(String),
    LoggerInitError(String),
}

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            MainError::ConfigLoadError(ref msg) => {
                write!(f, "Unable to load config file: {}", msg)
            }
            MainError::LoggerInitError(ref msg) => {
                write!(f, "Unable to initialize logger: {}", msg)
            }
        }
    }
}

impl Error for MainError {}

fn jobs(config: config::Config) {
    let mut once_pull = true;
    let mut once_cleanup = true;
    let mut once_check = true;
    let mut once_refresh = true;

    let mut last_time_pull = Instant::now();
    let mut last_time_cleanup = Instant::now();
    let mut last_time_check = Instant::now();
    let mut last_time_refresh = Instant::now();

    let mut list_deleted: Vec<UuidWithTime> = vec![];
    let client = Client::new();

    thread::spawn(move || loop {
        if config.servers_pull.len() > 0
            && (once_pull
                || last_time_pull.elapsed().as_secs() >= config.mirror_pull_interval.as_secs())
        {
            once_pull = false;
            once_refresh = true;
            last_time_pull = Instant::now();
            let result = pull::pull_worker(
                &client,
                config.connection_string.clone(),
                &config.servers_pull,
                config.chunk_size_changes,
                config.chunk_size_checks,
                config.max_duplicates,
                &mut list_deleted,
            );
            match result {
                Ok(_) => {}
                Err(err) => {
                    error!("Error in pull worker: {}", err);
                }
            }
            // remove items from deleted list after 1 day
            list_deleted.retain(|item| item.instant.elapsed().as_secs() < 3600 * 24);
            debug!(
                "List of deleted station uuids (duplicates): len={}",
                list_deleted.len()
            );
        }

        if once_cleanup || last_time_cleanup.elapsed().as_secs() >= 3600 {
            once_cleanup = false;
            once_refresh = true;
            last_time_cleanup = Instant::now();
            let result = cleanup::do_cleanup(
                config.delete,
                config.connection_string.clone(),
                config.click_valid_timeout.as_secs(),
                config.broken_stations_never_working_timeout.as_secs(),
                config.broken_stations_timeout.as_secs(),
                config.checks_timeout.as_secs(),
                config.clicks_timeout.as_secs(),
            );
            if let Err(error) = result {
                error!("Error: {}", error);
            }
        }

        if config.enable_check
            && (once_check || last_time_check.elapsed().as_secs() >= config.pause.as_secs())
        {
            trace!(
                "Check started.. (concurrency: {}, chunksize: {})",
                config.concurrency,
                config.check_stations
            );
            once_check = false;
            once_refresh = true;
            last_time_check = Instant::now();
            let result = check::dbcheck(
                config.connection_string.clone(),
                &config.source,
                config.concurrency,
                config.check_stations,
                config.tcp_timeout.as_secs(),
                config.max_depth,
                config.retries,
                config.check_servers,
            );
            match result {
                Ok(_) => {}
                Err(err) => {
                    error!("Check worker error: {}", err);
                }
            }

            if config.check_servers {
                let result = checkserver::do_check(
                    config.connection_string.clone(),
                    config.check_servers_chunksize,
                    config.concurrency,
                );
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Check worker error: {}", err);
                    }
                }
            }
        }

        if config.update_caches_interval.as_secs() > 0
            && (once_refresh
                || last_time_refresh.elapsed().as_secs() >= config.update_caches_interval.as_secs())
        {
            once_refresh = false;
            last_time_refresh = Instant::now();
            let result = refresh::refresh_all_caches(config.connection_string.clone());
            match result {
                Ok(_) => {}
                Err(err) => {
                    error!("Refresh worker error: {}", err);
                }
            }
        }

        thread::sleep(Duration::from_secs(10));
    });
}

fn mainloop() -> Result<(), Box<dyn Error>> {
    let config = config::load_config().map_err(|e| MainError::ConfigLoadError(e.to_string()))?;
    logger::setup_logger(config.log_level, &config.log_dir, config.log_json)
        .map_err(|e| MainError::LoggerInitError(e.to_string()))?;

    info!("Config: {:#?}", config);

    loop {
        let connection = db::MysqlConnection::new(&config.connection_string);
        match connection {
            Ok(connection) => {
                let migration_result = connection.do_migrations(
                    config.ignore_migration_errors,
                    config.allow_database_downgrade,
                );
                match migration_result {
                    Ok(_) => {
                        jobs(config.clone());
                        api::start(connection, config);
                    }
                    Err(err) => {
                        error!("Migrations error: {}", err);
                        thread::sleep(time::Duration::from_millis(1000));
                    }
                };
                break;
            }
            Err(e) => {
                error!("DB connection error: {}", e);
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
    Ok(())
}

fn main() {
    match mainloop() {
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
        Ok(_) => {
            std::process::exit(0);
        }
    }
}
