#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate url;
extern crate handlebars;

#[macro_use]
extern crate mysql;

use clap::{App, Arg};

use std::{thread, time};

mod api;
mod db;
mod simple_migrate;
mod pull_servers;
mod api_error;

fn main() {
    let matches = App::new("stream-check")
        .version(crate_version!())
        .author("segler_alex@web.de")
        .about("HTTP Rest API for radiobrowser")
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .value_name("DATABASE_URL")
                .help("Database connection url")
                .env("DATABASE_URL")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("listen_host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .help("listening host ip")
                .env("HOST")
                .default_value("127.0.0.1")
                .takes_value(true),
        ).arg(
            Arg::with_name("server_url")
                .short("s")
                .long("server_url")
                .value_name("SERVER_URL")
                .help("full server url that should be used in docs")
                .env("SERVER_URL")
                .default_value("localhost:8080")
                .takes_value(true),
        ).arg(
            Arg::with_name("listen_port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("listening port")
                .env("PORT")
                .default_value("8080")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .value_name("THREADS")
                .help("concurrent threads used by socket")
                .env("THREADS")
                .default_value("1")
                .takes_value(true),
        ).arg(
            Arg::with_name("mirror")
                .short("m")
                .long("mirror")
                .value_name("MIRROR")
                .help("address of other radiobrowser server to pull updates from")
                .multiple(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("update-caches-interval")
                .short("u")
                .long("update-caches-interval")
                .value_name("UPDATE_CACHES_INTERVAL")
                .help("update caches at an interval in seconds")
                .env("UPDATE_CACHES_INTERVAL")
                .default_value("0")
                .takes_value(true),
        ).arg(
            Arg::with_name("mirror-pull-interval")
                .short("q")
                .long("mirror-pull-interval")
                .value_name("MIRROR_PULL_INTERVAL")
                .help("pull from mirrors at an interval in seconds")
                .env("MIRROR_PULL_INTERVAL")
                .default_value("30")
                .takes_value(true),
        ).arg(
            Arg::with_name("ignore-migration-errors")
                .short("i")
                .long("ignore-migration-errors")
                .value_name("IGNORE_MIGRATION_ERRORS")
                .takes_value(false)
                .help("ignore errors in migrations"),
        ).arg(
            Arg::with_name("allow-database-downgrade")
                .short("a")
                .long("allow-database-downgrade")
                .value_name("IGNORE_MIGRATION_ERRORS")
                .takes_value(false)
                .help("allows downgrade of database if tables were created with newer software version"),
        ).arg(
            Arg::with_name("static-files-dir")
                .short("g")
                .long("static-files-dir")
                .value_name("STATIC_FILES_DIR")
                .help("directory that contains the static files")
                .env("STATIC_FILES_DIR")
                .default_value("./static/")
                .takes_value(true),
        ).get_matches();

    let connection_string: String = matches.value_of("database").unwrap().to_string();
    let static_files_dir: String = matches.value_of("static-files-dir").unwrap().to_string();
    let listen_host: String = matches.value_of("listen_host").unwrap().parse().expect("listen_host is not string");
    let listen_port: i32 = matches.value_of("listen_port").unwrap().parse().expect("listen_port is not u32");
    let server_url: &str = matches.value_of("server_url").unwrap();
    let threads: usize = matches.value_of("threads").unwrap().parse().expect("threads is not usize");
    let update_caches_interval: u64 = matches.value_of("update-caches-interval").unwrap().parse().expect("update-caches-interval is not u64");
    let mirror_pull_interval: u64 = matches.value_of("mirror-pull-interval").unwrap().parse().expect("update-caches-interval is not u64");
    let ignore_migration_errors: bool = matches.occurrences_of("ignore-migration-errors") > 0;
    let allow_database_downgrade: bool = matches.occurrences_of("allow-database-downgrade") > 0;

    let mut servers_pull = vec![];
    let mirrors = matches.values_of("mirror");
    if let Some(mirrors) = mirrors {
        for mirror in mirrors {
            println!("Will pull from '{}'", mirror);
            servers_pull.push(mirror.to_string());
        }
    }

    loop {
        let connection = db::new(&connection_string, update_caches_interval, ignore_migration_errors, allow_database_downgrade);
        match connection {
            Ok(v) => {
                api::run(v, listen_host, listen_port, threads, server_url, &static_files_dir, servers_pull, mirror_pull_interval);
                break;
            }
            Err(e) => {
                println!("{}", e);
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
}
