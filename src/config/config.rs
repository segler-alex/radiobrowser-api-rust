use std::time::Duration;

#[derive(Debug, Clone)]
pub enum CacheType {
    None,
    BuiltIn,
    Redis,
    Memcached,
}

impl From<CacheType> for String {
    fn from(c: CacheType) -> Self {
        match c {
            CacheType::None => String::from("none"),
            CacheType::BuiltIn => String::from("builtin"),
            CacheType::Redis => String::from("redis"),
            CacheType::Memcached => String::from("memcached"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OauthServer {
    pub id: String,
    pub name: String,
    pub icon_url: String,
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: String,
    pub email_url: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub allow_database_downgrade: bool,
    pub broken_stations_never_working_timeout: Duration,
    pub broken_stations_timeout: Duration,
    pub check_stations: u32,
    pub checks_timeout: Duration,
    pub click_valid_timeout: Duration,
    pub clicks_timeout: Duration,
    pub concurrency: usize,
    pub connection_string: String,
    pub delete: bool,
    pub enable_check: bool,
    pub ignore_migration_errors: bool,
    pub listen_host: String,
    pub listen_port: i32,
    pub log_dir: String,
    pub log_level: usize,
    pub log_json: bool,
    pub max_depth: u8,
    pub mirror_pull_interval: Duration,
    pub pause: Duration,
    pub prometheus_exporter_prefix: String,
    pub prometheus_exporter: bool,
    pub retries: u8,
    pub server_url: String,
    pub servers_pull: Vec<String>,
    pub source: String,
    pub server_location: String,
    pub server_country_code: String,
    pub static_files_dir: String,
    pub tcp_timeout: Duration,
    pub threads: usize,
    pub update_caches_interval: Duration,
    pub useragent: String,
    pub cache_type: CacheType,
    pub cache_url: String,
    pub cache_ttl: Duration,
    pub chunk_size_changes: usize,
    pub chunk_size_checks: usize,
    pub max_duplicates: usize,
    pub check_servers: bool,
    pub check_servers_chunksize: u32,
    pub language_replace_filepath: String,
    pub language_to_code_filepath: String,
    pub tag_replace_filepath: String,
    pub enable_extract_favicon: bool,
    pub recheck_existing_favicon: bool,
    pub favicon_size_min: usize,
    pub favicon_size_max: usize,
    pub favicon_size_optimum: usize,
    pub refresh_config_interval: Duration,
    pub cleanup_interval: Duration,
    pub oauth_servers: Vec<OauthServer>,
}
