#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate handlebars;
extern crate url;
#[macro_use]
extern crate mysql;
extern crate toml;
#[macro_use]
extern crate log;

extern crate humantime;
extern crate uuid;

extern crate av_stream_info_rust;
extern crate colored;
extern crate hostname;
extern crate native_tls;
extern crate reqwest;
extern crate threadpool;
extern crate website_icon_extract;
use std::{thread, time};

mod api;
mod check;
mod cleanup;
mod config;
mod db;
mod pull;
mod refresh;

fn main() {
    env_logger::init();

    let config = config::load_config().expect("Unable to load config file");

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
                        let click_valid_timeout: std::time::Duration = config.click_valid_timeout.into();

                        refresh::start(
                            config.connection_string.clone(),
                            config.update_caches_interval.as_secs(),
                        );
                        pull::start(
                            config.connection_string.clone(),
                            config.servers_pull,
                            config.mirror_pull_interval.as_secs(),
                        );
                        cleanup::start(
                            config.connection_string.clone(),
                            config.delete,
                            3600,
                            click_valid_timeout.as_secs(),
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
                            config.useragent,
                            config.tcp_timeout.as_secs(),
                            config.max_depth,
                            config.retries,
                            config.favicon,
                            config.enable_check,
                            config.pause.as_secs(),
                        );

                        api::start(
                            connection,
                            config.listen_host,
                            config.listen_port,
                            config.threads,
                            &config.server_url,
                            &config.static_files_dir,
                            &config.log_dir,
                            config.prometheus_exporter,
                            &config.prometheus_exporter_prefix,
                            click_valid_timeout.as_secs(),
                        );
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
}
