use crate::api::api_response::ApiResponse;
use crate::config::Config;
use crate::config::OauthServer;
use std::error::Error;
use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiConfig {
    pub check_enabled: bool,
    pub prometheus_exporter_enabled: bool,
    pub pull_servers: Vec<String>,
    pub tcp_timeout_seconds: u64,
    pub broken_stations_never_working_timeout_seconds: u64,
    pub broken_stations_timeout_seconds: u64,
    pub checks_timeout_seconds: u64,
    pub click_valid_timeout_seconds: u64,
    pub clicks_timeout_seconds: u64,
    pub mirror_pull_interval_seconds: u64,
    pub update_caches_interval_seconds: u64,
    pub server_name: String,
    pub server_location: String,
    pub server_country_code: String,
    pub check_retries: u8,
    pub check_batchsize: u32,
    pub check_pause_seconds: u64,
    pub api_threads: usize,
    pub cache_type: String,
    pub cache_ttl: u64,
    pub language_replace_filepath: String,
    pub language_to_code_filepath: String,
    pub oauth_servers: Vec<ApiConfigOauthServer>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiConfigOauthServer {
    pub id: String,
    pub name: String,
    pub icon_url: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: String,
    pub email_url: String,
}

impl ApiConfig {
    pub fn serialize_config(config: ApiConfig) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("config")?;
        xml.elem_text("check_enabled", &config.check_enabled.to_string())?;
        xml.elem_text(
            "prometheus_exporter_enabled",
            &config.prometheus_exporter_enabled.to_string(),
        )?;
        {
            xml.begin_elem("pull_servers")?;
            for server in config.pull_servers {
                xml.elem_text("url", &server)?;
            }
            xml.end_elem()?;
        }
        xml.elem_text(
            "tcp_timeout_seconds",
            &config.tcp_timeout_seconds.to_string(),
        )?;
        xml.elem_text(
            "broken_stations_never_working_timeout_seconds",
            &config
                .broken_stations_never_working_timeout_seconds
                .to_string(),
        )?;
        xml.elem_text(
            "broken_stations_timeout_seconds",
            &config.broken_stations_timeout_seconds.to_string(),
        )?;
        xml.elem_text(
            "checks_timeout_seconds",
            &config.checks_timeout_seconds.to_string(),
        )?;
        xml.elem_text(
            "click_valid_timeout_seconds",
            &config.click_valid_timeout_seconds.to_string(),
        )?;
        xml.elem_text(
            "clicks_timeout_seconds",
            &config.clicks_timeout_seconds.to_string(),
        )?;
        xml.elem_text(
            "mirror_pull_interval_seconds",
            &config.mirror_pull_interval_seconds.to_string(),
        )?;
        xml.elem_text(
            "update_caches_interval_seconds",
            &config.update_caches_interval_seconds.to_string(),
        )?;
        xml.elem_text("server_name", &config.server_name)?;
        xml.elem_text("server_location", &config.server_location)?;
        xml.elem_text("server_country_code", &config.server_country_code)?;

        xml.elem_text("check_retries", &config.check_retries.to_string())?;
        xml.elem_text("check_batchsize", &config.check_batchsize.to_string())?;
        xml.elem_text(
            "check_pause_seconds",
            &config.check_pause_seconds.to_string(),
        )?;
        xml.elem_text("api_threads", &config.api_threads.to_string())?;
        xml.elem_text("cache_type", &config.cache_type.to_string())?;
        xml.elem_text("cache_ttl", &config.cache_ttl.to_string())?;
        xml.elem_text("language_replace_filepath", &config.language_replace_filepath)?;
        xml.elem_text("language_to_code_filepath", &config.language_to_code_filepath)?;

        xml.begin_elem("oauth_servers")?;
        for oauthserver in config.oauth_servers {
            xml.begin_elem("server")?;
            xml.elem_text("id", &oauthserver.id)?;
            xml.elem_text("name", &oauthserver.name)?;
            xml.elem_text("icon_url", &oauthserver.icon_url)?;
            xml.elem_text("auth_url", &oauthserver.auth_url)?;
            xml.elem_text("token_url", &oauthserver.token_url)?;
            xml.elem_text("scopes", &oauthserver.scopes)?;
            xml.elem_text("email_url", &oauthserver.email_url)?;
            xml.end_elem()?;
        }
        xml.end_elem()?;

        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(config: ApiConfig, format: &str) -> Result<ApiResponse, Box<dyn Error>> {
        Ok(match format {
            "json" => ApiResponse::Text(serde_json::to_string(&config)?),
            "xml" => ApiResponse::Text(ApiConfig::serialize_config(config)?),
            _ => ApiResponse::UnknownContentType,
        })
    }
}

impl From<Config> for ApiConfig {
    fn from(mut item: Config) -> Self {
        ApiConfig {
            check_enabled: item.enable_check,
            prometheus_exporter_enabled: item.prometheus_exporter,
            pull_servers: item.servers_pull,
            tcp_timeout_seconds: item.tcp_timeout.as_secs(),
            broken_stations_never_working_timeout_seconds: item
                .broken_stations_never_working_timeout
                .as_secs(),
            broken_stations_timeout_seconds: item.broken_stations_timeout.as_secs(),
            checks_timeout_seconds: item.checks_timeout.as_secs(),
            click_valid_timeout_seconds: item.click_valid_timeout.as_secs(),
            clicks_timeout_seconds: item.clicks_timeout.as_secs(),
            mirror_pull_interval_seconds: item.mirror_pull_interval.as_secs(),
            update_caches_interval_seconds: item.update_caches_interval.as_secs(),
            server_name: item.source,
            check_retries: item.retries,
            check_batchsize: item.check_stations,
            check_pause_seconds: item.pause.as_secs(),
            api_threads: item.threads,
            cache_type: item.cache_type.into(),
            cache_ttl: item.cache_ttl.as_secs(),
            server_location: item.server_location,
            server_country_code: item.server_country_code,
            language_replace_filepath: item.language_replace_filepath,
            language_to_code_filepath: item.language_to_code_filepath,
            oauth_servers: item.oauth_servers.drain(..).map(|server| server.into()).collect(),
        }
    }
}

impl From<OauthServer> for ApiConfigOauthServer {
    fn from(item: OauthServer) -> Self {
        ApiConfigOauthServer {
            id: item.id,
            name: item.name,
            icon_url: item.icon_url,
            auth_url: item.auth_url,
            token_url: item.token_url,
            scopes: item.scopes,
            email_url: item.email_url,
        }
    }
}
