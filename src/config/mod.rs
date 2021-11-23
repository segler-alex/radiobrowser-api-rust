mod config;
mod config_error;
mod data_mapping_item;

use clap::{App, Arg};
pub use config::CacheType;
pub use config::Config;
pub use config_error::ConfigError;
use humantime;
use once_cell::sync::OnceCell;
use std::error::Error;
use std::fs;
use std::time::Duration;
use std::{collections::HashMap, sync::Mutex};

static INSTANCE_CONFIG: OnceCell<Mutex<Config>> = OnceCell::new();
static INSTANCE_LANGUAGE_TO_CODE: OnceCell<Mutex<HashMap<String, String>>> = OnceCell::new();
static INSTANCE_LANGUAGE_REPLACE: OnceCell<Mutex<HashMap<String, String>>> = OnceCell::new();
static INSTANCE_TAG_REPLACE: OnceCell<Mutex<HashMap<String, String>>> = OnceCell::new();

pub fn load_main_config() -> Result<(), Box<dyn Error>> {
    debug!("load_main_config()");
    match load_config() {
        Ok(config) => {
            trace!("Config: {:#?}", config);
            let m = INSTANCE_CONFIG.get_or_init(|| Mutex::new(config.clone()));
            *(m.lock()?) = config;
        }
        Err(err) => warn!("Unable to load file: {}", err),
    }
    Ok(())
}
pub fn load_all_extra_configs(c: &Config) -> Result<(), Box<dyn Error>> {
    match data_mapping_item::read_map_csv_file(&c.language_replace_filepath) {
        Ok(data) => {
            let m = INSTANCE_LANGUAGE_REPLACE.get_or_init(|| Mutex::new(data.clone()));
            *(m.lock()?) = data;
        }
        Err(err) => warn!(
            "Unable to load file '{}': {}",
            &c.language_replace_filepath, err
        ),
    }
    match data_mapping_item::read_map_csv_file(&c.tag_replace_filepath) {
        Ok(data) => {
            let m = INSTANCE_TAG_REPLACE.get_or_init(|| Mutex::new(data.clone()));
            *(m.lock()?) = data;
        }
        Err(err) => warn!(
            "Unable to load file '{}': {}",
            &c.tag_replace_filepath, err
        ),
    }
    match data_mapping_item::read_map_csv_file(&c.language_to_code_filepath) {
        Ok(data) => {
            let m = INSTANCE_LANGUAGE_TO_CODE.get_or_init(|| Mutex::new(data.clone()));
            *(m.lock()?) = data;
        }
        Err(err) => warn!(
            "Unable to load file '{}': {}",
            &c.language_to_code_filepath, err
        ),
    }
    Ok(())
}

pub fn get_config() -> Option<&'static Mutex<Config>> {
    INSTANCE_CONFIG.get()
}

pub fn get_cache_language_to_code() -> Option<&'static Mutex<HashMap<String, String>>> {
    INSTANCE_LANGUAGE_TO_CODE.get()
}

pub fn get_cache_language_replace() -> Option<&'static Mutex<HashMap<String, String>>> {
    INSTANCE_LANGUAGE_REPLACE.get()
}

pub fn get_cache_tags_replace() -> Option<&'static Mutex<HashMap<String, String>>> {
    INSTANCE_TAG_REPLACE.get()
}

pub fn convert_language_to_code<P: AsRef<str>>(language: P) -> Option<String> {
    // get global var
    match INSTANCE_LANGUAGE_TO_CODE.get() {
        Some(map) => {
            // get lock for mutex
            let map = map.lock();
            match map {
                Ok(map) => {
                    // search in hashmap
                    map.get(language.as_ref()).map(|item| item.to_string())
                }
                Err(_) => None,
            }
        }
        None => None,
    }
}

fn get_option_string(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: String,
) -> Result<String, Box<dyn Error>> {
    let value_from_clap = matches.value_of(setting_name);
    if let Some(value_from_clap) = value_from_clap {
        return Ok(value_from_clap.to_string());
    }

    let setting = config.get(setting_name);
    if let Some(setting) = setting {
        if setting.is_str() {
            let setting_decoded = setting.as_str();
            if let Some(setting_decoded) = setting_decoded {
                return Ok(String::from(setting_decoded));
            }
        } else {
            return Err(Box::new(ConfigError::TypeError(
                setting_name.into(),
                setting.to_string(),
            )));
        }
    }

    Ok(default_value)
}

fn get_option_duration(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: String,
) -> Result<Duration, Box<dyn Error>> {
    let s = get_option_string(matches, config, setting_name, default_value)?;
    Ok(s.parse::<humantime::Duration>()?.into())
}

fn get_option_number(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: i64,
) -> Result<i64, Box<dyn Error>> {
    let value_from_clap = matches.value_of(setting_name);
    if let Some(value_from_clap) = value_from_clap {
        return Ok(value_from_clap.to_string().parse()?);
    }

    let setting = config.get(setting_name);
    if let Some(setting) = setting {
        if setting.is_integer() {
            let setting_decoded = setting.as_integer();
            if let Some(setting_decoded) = setting_decoded {
                return Ok(setting_decoded);
            }
        } else {
            return Err(Box::new(ConfigError::TypeError(
                setting_name.into(),
                setting.to_string(),
            )));
        }
    }

    Ok(default_value)
}

fn get_option_number_occurences(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: usize,
) -> Result<usize, Box<dyn Error>> {
    let value_from_clap = matches.occurrences_of(setting_name) as usize;
    if value_from_clap > 0 {
        return Ok(value_from_clap);
    }

    let setting = config.get(setting_name);
    if let Some(setting) = setting {
        if setting.is_integer() {
            let setting_decoded = setting.as_integer();
            if let Some(setting_decoded) = setting_decoded {
                return Ok(setting_decoded as usize);
            }
        } else {
            return Err(Box::new(ConfigError::TypeError(
                setting_name.into(),
                setting.to_string(),
            )));
        }
    }

    Ok(default_value)
}

fn get_option_bool(
    matches: &clap::ArgMatches,
    config: &toml::Value,
    setting_name: &str,
    default_value: bool,
) -> Result<bool, Box<dyn Error>> {
    let value_from_clap = matches.value_of(setting_name);
    if let Some(value_from_clap) = value_from_clap {
        return Ok(value_from_clap.to_string().parse()?);
    }

    let setting = config.get(setting_name);
    if let Some(setting) = setting {
        if setting.is_bool() {
            let setting_decoded = setting.as_bool();
            if let Some(setting_decoded) = setting_decoded {
                return Ok(setting_decoded);
            }
        } else {
            return Err(Box::new(ConfigError::TypeError(
                setting_name.into(),
                setting.to_string(),
            )));
        }
    }

    Ok(default_value)
}

fn get_hosts_from_config(config: &toml::Value) -> Result<Vec<String>, Box<dyn Error>> {
    let mut list = vec![];
    let setting = config.get("pullservers");
    if let Some(setting) = setting {
        let setting_decoded = setting.as_table().ok_or(Box::new(ConfigError::TypeError(
            "pullservers".into(),
            setting.to_string(),
        )))?;
        for i in setting_decoded {
            let host = i.1.get("host");
            if let Some(host) = host {
                let host_str = host.as_str().ok_or(Box::new(ConfigError::TypeError(
                    "host".into(),
                    host.to_string(),
                )))?;
                list.push(host_str.to_string());
            }
        }
    }
    Ok(list)
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let hostname_str: String = hostname::get()
        .map(|os_string| os_string.to_string_lossy().into_owned())
        .unwrap_or("".to_string());

    let matches = App::new("radiobrowser-api-rust")
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
            Arg::with_name("replace-language-file")
                .long("replace-language-file")
                .value_name("REPLACE_LANGUAGE_FILE")
                .help("Path to csv file for language replacement")
                .env("REPLACE_LANGUAGE_FILE")
                .takes_value(true),
        ).arg(
            Arg::with_name("replace-tag-file")
                .long("replace-tag-file")
                .value_name("REPLACE_TAG_FILE")
                .help("Path to csv file for tag replacement")
                .env("REPLACE_TAG_FILE")
                .takes_value(true),
        ).arg(
            Arg::with_name("language-to-code-file")
                .long("language-to-code-file")
                .value_name("LANGUAGE_TO_CODE_FILE")
                .help("Path to csv file for language to code mapping")
                .env("LANGUAGE_TO_CODE_FILE")
                .takes_value(true),
        ).arg(
            Arg::with_name("log-json")
                .short("j")
                .long("log-json")
                .value_name("LOG_JSON")
                .takes_value(true)
                .help("Log in JSON format"),
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
            Arg::with_name("prometheus-exporter")
                .short("e")
                .long("prometheus-exporter")
                .value_name("PROMETHEUS_EXPORTER")
                .takes_value(true)
                .help("export statistics through a prometheus compatible exporter"),
        ).arg(
            Arg::with_name("prometheus-exporter-prefix")
                .long("prometheus-exporter-prefix")
                .value_name("PROMETHEUS_EXPORTER_PREFIX")
                .takes_value(true)
                .help("prefix for all exported values on /metrics"),
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
                .help("update caches at an interval")
                .env("UPDATE_CACHES_INTERVAL")
                .takes_value(true),
        ).arg(
            Arg::with_name("mirror-pull-interval")
                .short("q")
                .long("mirror-pull-interval")
                .value_name("MIRROR_PULL_INTERVAL")
                .help("pull from mirrors at an interval")
                .env("MIRROR_PULL_INTERVAL")
                .takes_value(true),
        ).arg(
            Arg::with_name("refresh-config-interval")
                .long("refresh-config-interval")
                .value_name("REFRESH_CONFIG_INTERVAL")
                .help("download / redownload config interval")
                .env("REFRESH_CONFIG_INTERVAL")
                .takes_value(true),
        ).arg(
            Arg::with_name("cleanup-interval")
                .long("cleanup-interval")
                .value_name("CLEANUP_INTERVAL")
                .help("cleanup tasks interval")
                .env("CLEANUP_INTERVAL")
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
                .value_name("ALLOW_DATABASE_DOWNGRADE")
                .takes_value(true)
                .help("allows downgrade of database if tables were created with newer software version"),
        ).arg(
            Arg::with_name("log-level")
                .short("v")
                .long("verbose")
                .value_name("LOG_LEVEL")
                .takes_value(false)
                .multiple(true)
                .help("increases the log level. can be specified mutliple times 0..4"),
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
            Arg::with_name("server-location")
                .long("server-location")
                .help("freeform location server string")
                .env("SERVERLOCATION")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server-country-code")
                .long("server-country-code")
                .help("2 letter country code for server location")
                .env("SERVERCOUNTRYCODE")
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
            Arg::with_name("cache-type")
                .long("cache-type")
                .value_name("CACHETYPE")
                .help("one of none,builtin,redis,memcached")
                .env("CACHETYPE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cache-url")
                .long("cache-url")
                .value_name("URL")
                .help("url to access cache server")
                .env("CACHEURL")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cache-ttl")
                .long("cache-ttl")
                .value_name("DURATION")
                .help("time to life for cache items")
                .env("CACHETTL")
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
            Arg::with_name("click-valid-timeout")
                .long("click_valid_timeout")
                .value_name("CLICK_VALID_TIMEOUT")
                .help("Possible clicks from the same IP. IPs are removed after this timespan.")
                .env("CLICK_VALID_TIMEOUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("broken-stations-never-working-timeout")
                .long("broken_stations_never_working_timeout")
                .value_name("BROKEN_STATIONS_NEVER_WORKING_TIMEOUT")
                .help("Broken streams are removed after this timespan, if they have never worked.")
                .env("BROKEN_STATIONS_NEVER_WORKING_TIMEOUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("broken-stations-timeout")
                .long("broken_stations_timeout")
                .value_name("BROKEN_STATIONS_TIMEOUT")
                .help("Broken streams are removed after this timespan.")
                .env("BROKEN_STATIONS_TIMEOUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("checks-timeout")
                .long("checks_timeout")
                .value_name("CHECKS_TIMEOUT")
                .help("Checks are removed after this timespan.")
                .env("CHECKS_TIMEOUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("clicks-timeout")
                .long("clicks_timeout")
                .value_name("CLICKS_TIMEOUT")
                .help("Clicks are removed after this timespan.")
                .env("CLICKS_TIMEOUT")
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
                .help("tcp connect/read timeout")
                .env("TCP_TIMEOUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pause")
                .long("pause")
                .value_name("PAUSE")
                .help("database check pauses")
                .env("PAUSE")
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
            Arg::with_name("chunk-size-changes")
                .long("chunk-size-changes")
                .value_name("CHUNK_SIZE_CHANGES")
                .help("chunk size for downloading changes")
                .env("CHUNK_SIZE_CHANGES")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("chunk-size-checks")
                .long("chunk-size-checks")
                .value_name("CHUNK_SIZE_CHECKS")
                .help("chunk size for downloading checks")
                .env("CHUNK_SIZE_CHECKS")
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
            Arg::with_name("max-duplicates")
                .long("max-duplicates")
                .value_name("MAX_DUPLICATES")
                .help("Maximum stations that have the same url")
                .env("MAX_DUPLICATES")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("favicon-size-min")
                .long("favicon-size-min")
                .value_name("FAVICON_SIZE_MIN")
                .help("Minimum (width or height) of favicons extracted")
                .env("FAVICON_SIZE_MIN")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("favicon-size-max")
                .long("favicon-size-max")
                .value_name("FAVICON_SIZE_MAX")
                .help("Maximum (width or height) of favicons extracted")
                .env("FAVICON_SIZE_MAX")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("favicon-size-optimum")
                .long("favicon-size-optimum")
                .value_name("FAVICON_SIZE_OPTIMUM")
                .help("Optimum size of favicons extracted")
                .env("FAVICON_SIZE_OPTIMUM")
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
            Arg::with_name("enable-extract-favicon")
                .long("enable-extract-favicon")
                .value_name("ENABLE_EXTRACT_FAVICON")
                .help("enable checking homepage for new icon")
                .env("ENABLE_EXTRACT_FAVICON")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("recheck-existing-favicon")
                .long("recheck-existing-favicon")
                .value_name("RECHECK_EXISTING_FAVICON")
                .help("recheck existing favicons")
                .env("RECHECK_EXISTING_FAVICON")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server-info-check")
                .long("server-info-check")
                .value_name("ENABLE_SERVER_CHECK")
                .help("enable server checks")
                .env("ENABLE_SERVER_CHECK")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server-info-check-chunksize")
                .long("server-info-check-chunksize")
                .value_name("ENABLE_SERVER_CHECK_CHUNKSIZE")
                .help("chunk size for server check")
                .env("ENABLE_SERVER_CHECK_CHUNKSIZE")
                .takes_value(true),
        ).get_matches();

    let config_file_path: String = matches
        .value_of("config-file")
        .unwrap_or("/etc/radiobrowser.toml")
        .to_string();

    let contents = fs::read_to_string(config_file_path)?;
    let config = toml::from_str::<toml::Value>(&contents)?;

    let connection_string = get_option_string(
        &matches,
        &config,
        "database",
        String::from("mysql://radiouser:password@localhost/radio"),
    )?;
    let static_files_dir: String = get_option_string(
        &matches,
        &config,
        "static-files-dir",
        String::from("./static/"),
    )?;
    let log_dir: String = get_option_string(&matches, &config, "log-dir", String::from("."))?;
    let listen_host: String =
        get_option_string(&matches, &config, "listen-host", String::from("127.0.0.1"))?;
    let listen_port: i32 = get_option_number(&matches, &config, "listen-port", 8080)? as i32;

    let prometheus_exporter: bool =
        get_option_bool(&matches, &config, "prometheus-exporter", true)?;
    let prometheus_exporter_prefix: String = get_option_string(
        &matches,
        &config,
        "prometheus-exporter-prefix",
        String::from("radio_browser_"),
    )?;

    let server_url: String = get_option_string(
        &matches,
        &config,
        "server-url",
        String::from("http://localhost"),
    )?;
    let threads: usize = get_option_number(&matches, &config, "threads", 1)? as usize;
    let update_caches_interval = get_option_duration(
        &matches,
        &config,
        "update-caches-interval",
        String::from("2mins"),
    )?;
    let mirror_pull_interval = get_option_duration(
        &matches,
        &config,
        "mirror-pull-interval",
        String::from("5mins"),
    )?;
    let refresh_config_interval = get_option_duration(
        &matches,
        &config,
        "refresh-config-interval",
        String::from("1day"),
    )?;
    let ignore_migration_errors: bool =
        get_option_bool(&matches, &config, "ignore-migration-errors", false)?;
    let allow_database_downgrade: bool =
        get_option_bool(&matches, &config, "allow-database-downgrade", false)?;
    let log_level: usize = get_option_number_occurences(&matches, &config, "log-level", 0)?;
    let log_json: bool = get_option_bool(&matches, &config, "log-json", false)?;
    let check_servers: bool = get_option_bool(&matches, &config, "server-info-check", false)?;
    let check_servers_chunksize =
        get_option_number(&matches, &config, "server-info-check-chunksize", 100)? as u32;

    let concurrency: usize = get_option_number(&matches, &config, "concurrency", 1)? as usize;
    let check_stations: u32 = get_option_number(&matches, &config, "stations", 10)? as u32;
    let enable_check: bool = get_option_bool(&matches, &config, "enable-check", false)?;
    let enable_extract_favicon: bool =
        get_option_bool(&matches, &config, "enable-extract-favicon", false)?;
    let recheck_existing_favicon: bool =
        get_option_bool(&matches, &config, "recheck-existing-favicon", false)?;

    let favicon_size_min: usize =
        get_option_number(&matches, &config, "favicon-size-min", 32)? as usize;
    let favicon_size_max: usize =
        get_option_number(&matches, &config, "favicon-size-max", 256)? as usize;
    let favicon_size_optimum: usize =
        get_option_number(&matches, &config, "favicon-size-optimum", 128)? as usize;

    let delete: bool = get_option_bool(&matches, &config, "delete", false)?;
    let pause = get_option_duration(&matches, &config, "pause", String::from("10secs"))?;
    let tcp_timeout =
        get_option_duration(&matches, &config, "tcp-timeout", String::from("10secs"))?;
    let max_depth: u8 = get_option_number(&matches, &config, "max-depth", 5)? as u8;
    let retries: u8 = get_option_number(&matches, &config, "retries", 5)? as u8;
    let source: String = get_option_string(&matches, &config, "source", hostname_str)?;
    let server_location: String =
        get_option_string(&matches, &config, "server-location", String::from(""))?;
    let server_country_code: String =
        get_option_string(&matches, &config, "server-country-code", String::from(""))?;
    let useragent = get_option_string(
        &matches,
        &config,
        "useragent",
        String::from("stream-check/0.1"),
    )?;
    let click_valid_timeout = get_option_duration(
        &matches,
        &config,
        "click-valid-timeout",
        String::from("1day"),
    )?;
    let broken_stations_never_working_timeout = get_option_duration(
        &matches,
        &config,
        "broken-stations-never-working-timeout",
        String::from("3days"),
    )?;
    let broken_stations_timeout = get_option_duration(
        &matches,
        &config,
        "broken-stations-timeout",
        String::from("30days"),
    )?;
    let cleanup_interval =
        get_option_duration(&matches, &config, "cleanup-interval", String::from("1hour"))?;
    let checks_timeout =
        get_option_duration(&matches, &config, "checks-timeout", String::from("30days"))?;
    let clicks_timeout =
        get_option_duration(&matches, &config, "clicks-timeout", String::from("30days"))?;

    let language_replace_filepath = get_option_string(
        &matches,
        &config,
        "replace-language-file",
        "language-replace.csv".to_string(),
    )?;
    let tag_replace_filepath = get_option_string(
        &matches,
        &config,
        "replace-tag-file",
        "tag-replace.csv".to_string(),
    )?;
    let language_to_code_filepath = get_option_string(
        &matches,
        &config,
        "language-to-code-file",
        "language-to-code.csv".to_string(),
    )?;

    let chunk_size_changes =
        get_option_number(&matches, &config, "chunk-size-changes", 999999)? as usize;
    let chunk_size_checks =
        get_option_number(&matches, &config, "chunk-size-checks", 999999)? as usize;

    let cache_type_str: String =
        get_option_string(&matches, &config, "cache-type", String::from("none"))?;
    let cache_url: String = get_option_string(&matches, &config, "cache-url", String::from(""))?;
    let cache_ttl = get_option_duration(&matches, &config, "cache-ttl", String::from("60secs"))?;
    let cache_type: CacheType = match cache_type_str.as_str() {
        "none" => Ok(CacheType::None),
        "builtin" => Ok(CacheType::BuiltIn),
        "redis" => Ok(CacheType::Redis),
        "memcached" => Ok(CacheType::Memcached),
        _ => Err(ConfigError::TypeError(
            "cache-type".into(),
            "possible values are none,builtin,redis,memcached".into(),
        )),
    }?;

    let max_duplicates = get_option_number(&matches, &config, "max-duplicates", 0)? as usize;
    let mut servers_pull = vec![];
    let mirrors = matches.values_of("mirror");
    if let Some(mirrors) = mirrors {
        for mirror in mirrors {
            servers_pull.push(mirror.to_string());
        }
    }

    let mut servers = get_hosts_from_config(&config)?;
    servers_pull.append(&mut servers);
    Ok(Config {
        allow_database_downgrade,
        broken_stations_never_working_timeout,
        broken_stations_timeout,
        check_stations,
        checks_timeout,
        click_valid_timeout,
        clicks_timeout,
        concurrency,
        connection_string,
        delete,
        enable_check,
        ignore_migration_errors,
        listen_host,
        listen_port,
        log_dir,
        log_level,
        log_json,
        max_depth,
        mirror_pull_interval,
        pause,
        prometheus_exporter_prefix,
        prometheus_exporter,
        retries,
        server_url,
        servers_pull,
        source,
        server_location,
        server_country_code,
        static_files_dir,
        tcp_timeout,
        threads,
        update_caches_interval,
        useragent,
        cache_type,
        cache_url,
        cache_ttl,
        chunk_size_changes,
        chunk_size_checks,
        max_duplicates,
        check_servers,
        check_servers_chunksize,
        language_replace_filepath,
        tag_replace_filepath,
        language_to_code_filepath,
        enable_extract_favicon,
        recheck_existing_favicon,
        favicon_size_min,
        favicon_size_max,
        favicon_size_optimum,
        refresh_config_interval,
        cleanup_interval,
    })
}
