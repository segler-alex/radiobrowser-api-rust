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

extern crate av_stream_info_rust;
extern crate colored;
extern crate hostname;
extern crate native_tls;
extern crate reqwest;
extern crate threadpool;
extern crate website_icon_extract;
use std::{thread, time};

mod api;
mod config;
mod check;
mod db;
mod refresh;
mod pull;
mod cleanup;

fn main() {
    env_logger::init();

    let config = config::load_config();

    info!("Config: {:#?}", config);

    loop {
        let connection_new = db::MysqlConnection::new(&config.connection_string);
        let connection = api::db::new(&config.connection_string);
        match connection {
            Ok(v) => {
                match connection_new {
                    Ok(v2) => {
                        let migration_result = v2.do_migrations(
                            config.ignore_migration_errors,
                            config.allow_database_downgrade,
                        );
                        match migration_result {
                            Ok(_) => {
                                refresh::start_refresh_worker(config.connection_string.clone(), config.update_caches_interval);
                                pull::start_pull_worker(config.connection_string.clone(), config.servers_pull, config.mirror_pull_interval);
                                cleanup::start(config.connection_string.clone(), config.source.clone(), config.delete, 3600);

                                check::start(
                                    config.connection_string,
                                    config.source,
                                    config.concurrency,
                                    config.check_stations,
                                    config.useragent,
                                    config.tcp_timeout as u32,
                                    config.max_depth,
                                    config.retries,
                                    config.favicon,
                                    config.enable_check,
                                    config.pause_seconds,
                                );
        
                                api::run(
                                    v,
                                    v2,
                                    config.listen_host,
                                    config.listen_port,
                                    config.threads,
                                    &config.server_url,
                                    &config.static_files_dir,
                                    &config.log_dir,
                                    config.prometheus_exporter,
                                    &config.prometheus_exporter_prefix,
                                );
                            }
                            Err(err) => {
                                error!("Migrations error: {}", err);
                                thread::sleep(time::Duration::from_millis(1000));
                            }
                        };
                    }
                    Err(e) => {
                        error!("DB connection error: {}", e);
                        thread::sleep(time::Duration::from_millis(1000));
                    }
                }
                break;
            }
            Err(e) => {
                error!("DB connection error: {}", e);
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
}
