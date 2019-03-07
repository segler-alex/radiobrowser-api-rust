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
#[macro_use(slog_o)]
extern crate slog;
extern crate slog_async;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
use slog::Drain;

use std::{thread, time};

mod api;
mod config;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog_o![]);

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    let config = config::load_config();
    trace!("Config: {:#?}", config);

    loop {
        let connection = api::db::new(
            &config.connection_string,
            config.update_caches_interval,
            config.ignore_migration_errors,
            config.allow_database_downgrade,
        );
        match connection {
            Ok(v) => {
                api::run(
                    v,
                    config.listen_host,
                    config.listen_port,
                    config.threads,
                    &config.server_url,
                    &config.static_files_dir,
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
