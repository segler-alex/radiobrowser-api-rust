#[macro_use]
extern crate clap;
#[macro_use]
extern crate mysql;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

use crate::db::DbConnection;
use crate::pull::UuidWithTime;
use core::fmt::Display;
use core::fmt::Formatter;
use reqwest::blocking::Client;
use signal_hook;
use signal_hook::consts::SIGHUP;
use signal_hook::iterator::Signals;
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
    LoggerInitError(String),
}

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            MainError::LoggerInitError(ref msg) => {
                write!(f, "Unable to initialize logger: {}", msg)
            }
        }
    }
}

impl Error for MainError {}

fn jobs<C: 'static>(conn: C)
where
    C: DbConnection + Clone + Send,
{
    let mut once_refresh_config = false;
    let mut once_pull = true;
    let mut once_cleanup = true;
    let mut once_check = true;
    let mut once_refresh_caches = true;

    let mut last_time_refresh_config = Instant::now();
    let mut last_time_pull = Instant::now();
    let mut last_time_cleanup = Instant::now();
    let mut last_time_check = Instant::now();
    let mut last_time_refresh_caches = Instant::now();

    let mut list_deleted: Vec<UuidWithTime> = vec![];
    let client = Client::new();

    thread::spawn(move || loop {
        let config = config::get_config()
            .expect("No config loaded")
            .lock()
            .expect("Config could not be pulled from shared memory.")
            .clone();
        
        if config.refresh_config_interval.as_secs() > 0
            && (once_refresh_config
                || last_time_refresh_config.elapsed().as_secs()
                    >= config.refresh_config_interval.as_secs())
        {
            once_refresh_config = false;
            last_time_refresh_config = Instant::now();
            match config::load_all_extra_configs(&config) {
                Ok(_) => {}
                Err(err) => {
                    error!("Reload config: {}", err);
                }
            }
        }

        if config.servers_pull.len() > 0
            && (once_pull
                || last_time_pull.elapsed().as_secs() >= config.mirror_pull_interval.as_secs())
        {
            once_pull = false;
            once_refresh_caches = true;
            last_time_pull = Instant::now();
            let result = pull::pull_worker(
                &client,
                conn.clone(),
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

        if config.cleanup_interval.as_secs() > 0
            && (once_cleanup
                || last_time_cleanup.elapsed().as_secs() >= config.cleanup_interval.as_secs())
        {
            once_cleanup = false;
            once_refresh_caches = true;
            last_time_cleanup = Instant::now();
            let result = cleanup::do_cleanup(
                config.delete,
                conn.clone(),
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
            once_refresh_caches = true;
            last_time_check = Instant::now();
            let result = check::dbcheck(
                conn.clone(),
                &config.source,
                config.concurrency,
                config.check_stations,
                config.tcp_timeout.as_secs(),
                config.max_depth,
                config.retries,
                config.check_servers,
                config.recheck_existing_favicon,
                config.enable_extract_favicon,
                config.favicon_size_min,
                config.favicon_size_max,
                config.favicon_size_optimum,
            );

            match result {
                Ok(_) => {}
                Err(err) => {
                    error!("Check worker error: {}", err);
                }
            }
            if config.check_servers {
                let result = checkserver::do_check(
                    conn.clone(),
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
            && (once_refresh_caches
                || last_time_refresh_caches.elapsed().as_secs()
                    >= config.update_caches_interval.as_secs())
        {
            once_refresh_caches = false;
            last_time_refresh_caches = Instant::now();
            let result = refresh::refresh_all_caches(conn.clone());
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
    // load config
    config::load_main_config()?;
    let config = {
        config::get_config()
            .expect("config could not be loaded")
            .lock()
            .expect("could not load config from shared mem")
            .clone()
    };
    logger::setup_logger(config.log_level, &config.log_dir, config.log_json)
        .map_err(|e| MainError::LoggerInitError(e.to_string()))?;
    info!("Config: {:#?}", config);
    config::load_all_extra_configs(&config)?;

    let config2 = config.clone();

    thread::spawn(|| loop {
        let connection = db::MysqlConnection::new(&config2.connection_string);
        match connection {
            Ok(connection) => {
                if config2.no_migrations {
                    match connection.migrations_needed() {
                        Ok(migrations_needed) => {
                            if !migrations_needed {
                                debug!("Migrations are not allowed but not needed.");
                                jobs(connection.clone());
                                api::start(connection, config2);
                            } else {
                                error!("Migrations are needed but not allowed by parameter!");
                                thread::sleep(time::Duration::from_millis(1000));
                            }
                        },
                        Err(err) => {
                            error!("Migrations checking error: {}", err);
                            thread::sleep(time::Duration::from_millis(1000));
                        }
                    }
                } else {
                    let migration_result = connection.do_migrations(
                        config2.ignore_migration_errors,
                        config2.allow_database_downgrade,
                    );
                    match migration_result {
                        Ok(_) => {
                            debug!("Migrations done.");
                            jobs(connection.clone());
                            api::start(connection, config2);
                        }
                        Err(err) => {
                            error!("Migrations error: {}", err);
                            thread::sleep(time::Duration::from_millis(1000));
                        }
                    };
                }
                break;
            }
            Err(e) => {
                error!("DB connection error: {}", e);
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    });

    let mut signals = Signals::new(&[SIGHUP])?;
    for signal in &mut signals {
        match signal {
            SIGHUP => {
                info!("received HUP, reload config");
                config::load_main_config()?;
                config::load_all_extra_configs(&config)?;
            }
            _ => unreachable!(),
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
