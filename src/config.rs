use clap::{App, Arg};
use std::fs;

use hostname::get_hostname;

#[derive(Debug)]
pub struct Config {
    pub listen_host: String,
    pub listen_port: i32,
    pub connection_string: String,
    pub update_caches_interval: u64,
    pub ignore_migration_errors: bool,
    pub allow_database_downgrade: bool,
    pub threads: usize,
    pub server_url: String,
    pub static_files_dir: String,
    pub log_dir: String,
    pub servers_pull: Vec<String>,
    pub mirror_pull_interval: u64,
    pub log_level: usize,

    pub concurrency: usize,
    pub check_stations: u32,
    pub enable_check: bool,
    pub delete: bool,
    pub favicon: bool,
    pub pause_seconds: u64,
    pub tcp_timeout: u64,
    pub max_depth: u8,
    pub retries: u8,
    pub source: String,
    pub useragent: String,
}

fn get_option_string(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: String,
) -> String {
    let value_from_clap = matches.value_of(setting_name);
    if let Some(value_from_clap) = value_from_clap {
        return value_from_clap.to_string();
    }

    let setting = config.get(setting_name);
    if let Some(setting) = setting {
        if setting.is_str() {
            let setting_decoded = setting.as_str();
            if let Some(setting_decoded) = setting_decoded {
                return String::from(setting_decoded);
            }
        }
    }

    default_value
}

fn get_option_number(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: i64,
) -> i64 {
    let value_from_clap = matches.value_of(setting_name);
    if let Some(value_from_clap) = value_from_clap {
        return value_from_clap.to_string().parse().unwrap_or(default_value);
    }

    let setting = config.get(setting_name);
    if let Some(setting) = setting {
        let setting_decoded = setting.as_integer();
        if let Some(setting_decoded) = setting_decoded {
            return setting_decoded;
        }
    }

    default_value
}

fn get_option_bool(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: bool,
) -> bool {
    let value_from_clap = matches.value_of(setting_name);
    if let Some(value_from_clap) = value_from_clap {
        return value_from_clap.to_string().parse().unwrap_or(default_value);
    }

    let setting = config.get(setting_name);
    if let Some(setting) = setting {
        let setting_decoded = setting.as_bool();
        if let Some(setting_decoded) = setting_decoded {
            return setting_decoded;
        }
    }

    default_value
}

fn get_hosts_from_config(config: &toml::Value) -> Vec<String> {
    let mut list = vec![];
    let setting = config.get("pullservers");
    if let Some(setting) = setting {
        let setting_decoded = setting.as_table();
        if let Some(setting_decoded) = setting_decoded {
            for i in setting_decoded {
                let host = i.1.get("host");
                if let Some(host) = host {
                    let host_str = host.as_str();
                    if let Some(host_str) = host_str {
                        list.push(host_str.to_string());
                    }
                }
            }
        }
    }
    list
}

pub fn load_config() -> Config {
    let hostname: String = get_hostname().unwrap_or("".to_string());

    let matches = App::new("stream-check")
        .version(crate_version!())
        .author("segler_alex@web.de")
        .about("HTTP Rest API for radiobrowser")
        .arg(
            Arg::with_name("config-file")
                .short("f")
                .long("config-file")
                .value_name("CONFIG-FILE")
                .help("Path to config file")
                .env("CONFIG_FILE")
                .default_value("/etc/radiobrowser.toml")
                .takes_value(true),
        ).arg(
            Arg::with_name("log-dir")
                .short("l")
                .long("log-dir")
                .value_name("LOG-DIR")
                .help("Path to log dir")
                .env("LOG_DIR")
                .takes_value(true),
        ).arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .value_name("DATABASE_URL")
                .help("Database connection url")
                .env("DATABASE_URL")
                .takes_value(true),
        ).arg(
            Arg::with_name("listen-host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .help("listening host ip")
                .env("HOST")
                .takes_value(true),
        ).arg(
            Arg::with_name("server-url")
                .short("s")
                .long("server-url")
                .value_name("SERVER_URL")
                .help("full server url that should be used in docs")
                .env("SERVER_URL")
                .takes_value(true),
        ).arg(
            Arg::with_name("listen-port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("listening port")
                .env("PORT")
                .takes_value(true),
        ).arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .value_name("THREADS")
                .help("concurrent threads used by socket")
                .env("THREADS")
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
                .takes_value(true),
        ).arg(
            Arg::with_name("mirror-pull-interval")
                .short("q")
                .long("mirror-pull-interval")
                .value_name("MIRROR_PULL_INTERVAL")
                .help("pull from mirrors at an interval in seconds")
                .env("MIRROR_PULL_INTERVAL")
                .takes_value(true),
        ).arg(
            Arg::with_name("ignore-migration-errors")
                .short("i")
                .long("ignore-migration-errors")
                .value_name("IGNORE_MIGRATION_ERRORS")
                .takes_value(true)
                .help("ignore errors in migrations"),
        ).arg(
            Arg::with_name("allow-database-downgrade")
                .short("a")
                .long("allow-database-downgrade")
                .value_name("IGNORE_MIGRATION_ERRORS")
                .takes_value(true)
                .help("allows downgrade of database if tables were created with newer software version"),
        ).arg(
            Arg::with_name("log-level")
                .short("v")
                .long("log-level")
                .value_name("LOG_LEVEL")
                .takes_value(false)
                .multiple(true)
                .help("increases the log level. can be specified mutliple times 0..3"),
        ).arg(
            Arg::with_name("static-files-dir")
                .short("g")
                .long("static-files-dir")
                .value_name("STATIC_FILES_DIR")
                .help("directory that contains the static files")
                .env("STATIC_FILES_DIR")
                .takes_value(true),
        ).arg(
            Arg::with_name("source")
                .long("source")
                .value_name("SOURCE")
                .help("Source string for database check entries")
                .env("SOURCE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("useragent")
                .long("useragent")
                .value_name("USERAGENT")
                .help("user agent value for http requests")
                .env("USERAGENT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("retries")
                .short("r")
                .long("retries")
                .value_name("RETRIES")
                .help("Max number of retries for station checks")
                .env("RETRIES")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("max-depth")
                .long("max_depth")
                .value_name("MAX_DEPTH")
                .help("max recursive link check depth")
                .env("MAX_DEPTH")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tcp-timeout")
                .long("tcp_timeout")
                .value_name("TCP_TIMEOUT")
                .help("tcp connect/read timeout in seconds")
                .env("TCP_TIMEOUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pause-seconds")
                .long("pause_seconds")
                .value_name("PAUSE_SECONDS")
                .help("database check pauses in seconds")
                .env("PAUSE_SECONDS")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("stations")
                .short("n")
                .long("stations")
                .value_name("STATIONS")
                .help("batch size for station checks")
                .env("STATIONS")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("concurrency")
                .short("c")
                .long("concurrency")
                .value_name("CONCURRENCY")
                .help("streams checked in parallel")
                .env("CONCURRENCY")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delete")
                .short("x")
                .long("delete")
                .value_name("DELETE")
                .help("delete broken stations according to rules")
                .env("DELETE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("enable-check")
                .long("enable-check")
                .value_name("ENABLE_CHECK")
                .help("enable station checks")
                .env("ENABLE_CHECK")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("favicon")
                .long("favicon")
                .value_name("FAVICON")
                .help("check favicons and try to repair them")
                .env("FAVICON")
                .takes_value(true),
        ).get_matches();

    let config_file_path: String = matches.value_of("config-file").unwrap().to_string();

    debug!("Load settings from file {}", config_file_path);
    let config = {
        let contents = fs::read_to_string(config_file_path);
        match contents {
            Ok(contents) => {
                let config = toml::from_str::<toml::Value>(&contents);
                match config {
                    Ok(config) => config,
                    Err(err) => {
                        panic!("Could not decode config file: {}", err);
                    }
                }
            }
            Err(err) => {
                error!("Could not load config file: {}", err);
                toml::from_str::<toml::Value>("").unwrap()
            }
        }
    };

    let connection_string = get_option_string(
        &matches,
        &config,
        "database",
        String::from("mysql://radiouser:password@localhost/radio"),
    );
    let static_files_dir: String = get_option_string(
        &matches,
        &config,
        "static-files-dir",
        String::from("./static/"),
    );
    let log_dir: String = get_option_string(&matches, &config, "log-dir", String::from("."));
    let listen_host: String =
        get_option_string(&matches, &config, "listen-host", String::from("127.0.0.1"));
    let listen_port: i32 = get_option_number(&matches, &config, "listen-port", 8080) as i32;
    let server_url: String = get_option_string(
        &matches,
        &config,
        "server-url",
        String::from("http://localhost"),
    );
    let threads: usize = get_option_number(&matches, &config, "threads", 1) as usize;
    let update_caches_interval: u64 =
        get_option_number(&matches, &config, "update-caches-interval", 0) as u64;
    let mirror_pull_interval: u64 =
        get_option_number(&matches, &config, "mirror-pull-interval", 30) as u64;
    let ignore_migration_errors: bool = get_option_bool(&matches, &config, "ignore-migration-errors", false);
    let allow_database_downgrade: bool = get_option_bool(&matches, &config, "allow-database-downgrade", false);
    let log_level: usize = matches.occurrences_of("log-level") as usize;

    let concurrency: usize = get_option_number(&matches, &config, "concurrency", 1) as usize;
    let check_stations: u32 = get_option_number(&matches, &config, "stations", 10) as u32;
    let enable_check: bool = get_option_bool(&matches, &config, "enable-check", false);
    let delete: bool = get_option_bool(&matches, &config, "delete", false);
    let favicon: bool = get_option_bool(&matches, &config, "favicon", false);
    let pause_seconds: u64 = get_option_number(&matches, &config, "pause-seconds", 10) as u64;
    let tcp_timeout: u64 = get_option_number(&matches, &config, "tcp-timeout", 10) as u64;
    let max_depth: u8 = get_option_number(&matches, &config, "max-depth", 5) as u8;
    let retries: u8 = get_option_number(&matches, &config, "retries", 5) as u8;
    let source: String = get_option_string(&matches, &config, "source", hostname);
    let useragent = get_option_string(&matches, &config, "useragent", String::from("stream-check/0.1"));
    let mut servers_pull = vec![];
    let mirrors = matches.values_of("mirror");
    if let Some(mirrors) = mirrors {
        for mirror in mirrors {
            info!("Will pull from '{}'", mirror);
            servers_pull.push(mirror.to_string());
        }
    }

    let mut servers = get_hosts_from_config(&config);
    servers_pull.append(&mut servers);

    Config {
        listen_host,
        listen_port,
        connection_string,
        update_caches_interval,
        ignore_migration_errors,
        allow_database_downgrade,
        mirror_pull_interval,
        servers_pull,
        threads,
        server_url,
        static_files_dir,
        log_dir,
        log_level,
        concurrency,
        check_stations,
        enable_check,
        delete,
        favicon,
        pause_seconds,
        tcp_timeout,
        max_depth,
        retries,
        source,
        useragent,
    }
}
