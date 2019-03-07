use clap::{App, Arg};
use std::fs;

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
    pub servers_pull: Vec<String>,
    pub mirror_pull_interval: u64,
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
    let matches = App::new("stream-check")
        .version(crate_version!())
        .author("segler_alex@web.de")
        .about("HTTP Rest API for radiobrowser")
        .arg(
            Arg::with_name("config-file")
                .short("c")
                .long("config-file")
                .value_name("CONFIG-FILE")
                .help("Path to config file")
                .env("CONFIG_FILE")
                .default_value("/etc/radiobrowser.toml")
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
            Arg::with_name("listen_host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .help("listening host ip")
                .env("HOST")
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
    let listen_host: String =
        get_option_string(&matches, &config, "listen_host", String::from("127.0.0.1"));
    let listen_port: i32 = get_option_number(&matches, &config, "listen_port", 8080) as i32;
    let server_url: String = get_option_string(&matches, &config, "server_url", String::from(""));
    let threads: usize = get_option_number(&matches, &config, "threads", 1) as usize;
    let update_caches_interval: u64 =
        get_option_number(&matches, &config, "update-caches-interval", 0) as u64;
    let mirror_pull_interval: u64 =
        get_option_number(&matches, &config, "mirror-pull-interval", 30) as u64;
    let ignore_migration_errors: bool = matches.occurrences_of("ignore-migration-errors") > 0;
    let allow_database_downgrade: bool = matches.occurrences_of("allow-database-downgrade") > 0;

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
    }
}
