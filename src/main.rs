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

fn main() {
    env_logger::init();

    let config = config::load_config();

    info!("Config: {:#?}", config);

    loop {
        let connection = api::db::new(
            &config.connection_string,
            config.update_caches_interval,
            config.ignore_migration_errors,
            config.allow_database_downgrade,
        );
        match connection {
            Ok(v) => {
                check::start(
                    config.connection_string,
                    config.source,
                    config.delete,
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
                    config.listen_host,
                    config.listen_port,
                    config.threads,
                    &config.server_url,
                    &config.static_files_dir,
                    &config.log_dir,
                    config.servers_pull,
                    config.mirror_pull_interval,
                );
                break;
            }
            Err(e) => {
                error!("{}", e);
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
}
