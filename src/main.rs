#[macro_use]
extern crate clap;
#[macro_use]
extern crate mysql;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

use core::fmt::Display;
use core::fmt::Formatter;
use std::error::Error;
use std::{thread, time};

mod api;
mod check;
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
                        let config_for_api = config.clone();

                        refresh::start(
                            config.connection_string.clone(),
                            config.update_caches_interval.as_secs(),
                        );
                        pull::start(
                            config.connection_string.clone(),
                            config.servers_pull,
                            config.mirror_pull_interval.as_secs(),
                            config.chunk_size_changes,
                            config.chunk_size_checks,
                            config.max_duplicates,
                        );
                        cleanup::start(
                            config.connection_string.clone(),
                            config.delete,
                            3600,
                            config.click_valid_timeout.as_secs(),
                            config.broken_stations_never_working_timeout.as_secs(),
                            config.broken_stations_timeout.as_secs(),
                            config.checks_timeout.as_secs(),
                            config.clicks_timeout.as_secs(),
                        );
                        check::start(
                            config.connection_string,
                            config.source,
                            config.concurrency,
                            config.check_stations,
                            config.tcp_timeout.as_secs(),
                            config.max_depth,
                            config.retries,
                            config.enable_check,
                            config.pause.as_secs(),
                        );

                        api::start(connection, config_for_api);
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
