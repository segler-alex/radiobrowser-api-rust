extern crate rouille;

extern crate serde;
extern crate serde_json;
extern crate dns_lookup;

pub mod data;
mod parameters;
mod prometheus_exporter;
mod api_error;
mod api_response;
mod cache;
mod all_params;

//use std::thread::JoinHandle;
//use rouille::Server;
use crate::api::data::ApiLanguage;
use all_params::AllParameters;
use prometheus_exporter::RegistryLinks;

use api_response::ApiResponse;

use std::error::Error;
use std::convert::TryInto;
use std::thread;
use std::time::Duration;
use api_error::ApiError;

use self::parameters::RequestParameters;

use crate::api::data::ApiStreamingServer;
use crate::api::data::ResultMessage;
use crate::api::data::StationCachedInfo;
use crate::api::data::StationHistoryCurrent;
use crate::api::data::Station;
use crate::api::data::StationCheck;
use crate::api::data::StationCheckStep;
use crate::api::data::StationAddResult;
use crate::api::data::Status;
use crate::api::data::StationClick;
use crate::api::data::ApiConfig;
use crate::db::DbConnection;
use crate::db::models::ExtraInfo;
use crate::db::models::State;
use crate::db::models::DbStationItem;
use crate::api::rouille::Response;
use crate::api::rouille::Request;
use std;
use self::dns_lookup::lookup_host;
use self::dns_lookup::lookup_addr;

use crate::config;
use crate::config::Config;

use std::fs::File;
use self::serde_json::value::{Map};
use serde::{Serialize,Deserialize};

use handlebars::{
    to_json, Handlebars,
};

#[derive(Serialize, Deserialize)]
pub struct ServerEntry {
    ip: String,
    name: String
}

fn add_cors(result : rouille::Response) -> rouille::Response {
    result.with_unique_header("Access-Control-Allow-Origin", "*")
        .with_unique_header("Access-Control-Allow-Headers", "origin, x-requested-with, content-type, User-Agent")
        .with_unique_header("Access-Control-Allow-Methods", "GET,POST")
}

fn get_only_first_item(mut stations: Vec<DbStationItem>) -> Option<DbStationItem>{
    if stations.len() == 1 {
        Some(stations.pop().unwrap())
    } else {
        None
    }
}

fn dns_resolve(format : &str) -> Result<ApiResponse, Box<dyn Error>> {
    let hostname = "all.api.radio-browser.info";
    let ips: Vec<std::net::IpAddr> = lookup_host(hostname)?;
    let mut list: Vec<ServerEntry> = Vec::new();
    for ip in ips {
        let ip_str : String = format!("{}",ip);
        let name : String = lookup_addr(&ip)?;
        let item = ServerEntry{ip: ip_str, name};
        list.push(item);
    }

    match format {
        "json" => Ok(ApiResponse::Text(serde_json::to_string(&list)?)),
        _ => Ok(ApiResponse::NotFound)
    }
}

fn encode_changes(list : Vec<StationHistoryCurrent>, format : &str) -> Result<ApiResponse, Box<dyn Error>> {
    Ok(match format {
        "csv" => {
            ApiResponse::Text(StationHistoryCurrent::serialize_changes_list_csv(list)?)
        },
        "json" => {
            ApiResponse::Text(serde_json::to_string(&list)?)
        },
        "xml" => {
            ApiResponse::Text(StationHistoryCurrent::serialize_changes_list(list)?)
        },
        _ => ApiResponse::UnknownContentType
    })
}

fn encode_message(status: Result<String, Box<dyn Error>>, format : &str) -> Result<ApiResponse, Box<dyn Error>> {
    Ok(match format {
        "json" => {
            match status {
                Ok(message) => ApiResponse::Text(serde_json::to_string(&ResultMessage::new(true,message))?),
                Err(err) => ApiResponse::Text(serde_json::to_string(&ResultMessage::new(false,err.to_string()))?),
            }
        },
        "xml" => {
            match status {
                Ok(message) => ApiResponse::Text(ResultMessage::new(true,message).serialize_xml()?),
                Err(err) => ApiResponse::Text(ResultMessage::new(false,err.to_string()).serialize_xml()?),
            }
        },
        _ => ApiResponse::UnknownContentType
    })
}

fn encode_station_url<A>(connection_new: &A, station: Option<DbStationItem>, ip: &str, format : &str, seconds: u64, registry: RegistryLinks) -> Result<ApiResponse, Box<dyn Error>> where A: DbConnection {
    Ok(match station {
        Some(station) => {
            registry.clicks.inc();
            let _ = connection_new.increase_clicks(&ip, &station, seconds);
            let station = station.into();
            match format {
                "json" => {
                    let s = Station::extract_cached_info(station, "retrieved station url");
                    ApiResponse::Text(serde_json::to_string(&s)?)
                },
                "xml" => {
                    let s = Station::extract_cached_info(station, "retrieved station url");
                    ApiResponse::Text(StationCachedInfo::serialize_cached_info(s)?)
                },
                "m3u" => {
                    let list = vec![station];
                    ApiResponse::Text(Station::serialize_to_m3u(list, true))
                },
                "pls" => {
                    let list = vec![station];
                    ApiResponse::Text(Station::serialize_to_pls(list, true))
                },
                _ => ApiResponse::UnknownContentType
            }
        },
        _ => ApiResponse::NotFound
    })
}

fn encode_states(list : Vec<State>, format : &str) -> Result<ApiResponse, Box<dyn Error>> {
    Ok(match format {
        "csv" => {
            ApiResponse::Text(State::serialize_state_list_csv(list)?)
        },
        "json" => {
            ApiResponse::Text(serde_json::to_string(&list)?)
        },
        "xml" => {
            ApiResponse::Text(State::serialize_state_list(list)?)
        },
        _ => ApiResponse::UnknownContentType
    })
}

impl From<config::CacheType> for cache::GenericCacheType {
    fn from(cache_type: config::CacheType) -> Self {
        match cache_type {
            config::CacheType::None => cache::GenericCacheType::None,
            config::CacheType::BuiltIn => cache::GenericCacheType::BuiltIn,
            config::CacheType::Redis => cache::GenericCacheType::Redis,
            config::CacheType::Memcached => cache::GenericCacheType::Memcached,
        }
    }
}

fn encode_extra(list : Vec<ExtraInfo>, format : &str, tag_name: &str) -> Result<ApiResponse, Box<dyn Error>> {
    Ok(match format {
        "csv" => {
            ApiResponse::Text(ExtraInfo::serialize_extra_list_csv(list)?)
        },
        "json" => {
            ApiResponse::Text(serde_json::to_string(&list)?)
        },
        "xml" => {
            ApiResponse::Text(ExtraInfo::serialize_extra_list(list, tag_name)?)
        },
        _ => ApiResponse::UnknownContentType
    })
}

fn encode_status(status: Status, format : &str, static_dir: &str) -> ApiResponse {
    match format {
        "json" => {
            let j = serde_json::to_string(&status);
            match j {
                Ok(j) => ApiResponse::Text(j),
                Err(err) => {
                    error!("Unable to serialize object to JSON {}",err);
                    ApiResponse::ServerError("Unable to serialize object to JSON".to_string())
                },
            }
        },
        "xml" => {
            let j = status.serialize_xml();
            match j {
                Ok(j) => ApiResponse::Text(j),
                Err(err) => {
                    error!("Unable to serialize object to XML {}",err);
                    ApiResponse::ServerError("Unable to serialize object to XML".to_string())
                },
            }
        },
        "html" => {
            let mut handlebars = Handlebars::new();
            let y = handlebars.register_template_file("stats.hbs", &format!("{}/{}",static_dir,"stats.hbs"));
            if y.is_ok(){
                let mut data = Map::new();
                data.insert(String::from("status"), to_json(status));
                let rendered = handlebars.render("stats.hbs", &data);
                match rendered {
                    Ok(rendered) => ApiResponse::Text(rendered),
                    Err(err) => {
                        error!("Unable to render HTML {}",err);
                        ApiResponse::ServerError("Unable to render HTML".to_string())
                    },
                }
            }else{
                error!("unable register template file: stats.hbs");
                ApiResponse::ServerError("unable to send stats".to_string())
            }
        },
        _ => ApiResponse::UnknownContentType
    }
}

/*
pub fn start_unavailable<F, T>(config: Config, func: F) -> JoinHandle<T> where
    F: FnOnce(std::sync::mpsc::Sender<()>) -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let server = Server::new(format!("{}:{}", &config.listen_host, &config.listen_port), |_| {
        debug!("received request while loading");
        // send "http service unavailable 503"
        Response::text("loading").with_status_code(503)
    }).unwrap();
    info!("Listen on {:?} (warming up)", server.server_addr());
    let (handle, sender) = server.stoppable();

    let thread_handle = thread::spawn(|| func(sender));
    handle.join().unwrap();

    thread_handle
}
*/

pub fn start<A: 'static +  std::clone::Clone>(
    connection_new: A,
    config: Config,
) where A: DbConnection, A: std::marker::Send, A: std::marker::Sync {
    let listen_str = format!("{}:{}", config.listen_host, config.listen_port);
    info!("Listen on {} with {} threads", listen_str, config.threads);

    let registry = prometheus_exporter::create_registry(&config.prometheus_exporter_prefix);
    if let Ok(registry) = registry {
        let cache = cache::GenericCache::new(config.cache_type.clone().into(), config.cache_url.clone(), config.cache_ttl.as_secs().try_into().expect("cache-ttl is too high"));

        if cache.needs_cleanup() {
            let mut cache_cleanup = cache.clone();
            thread::spawn(move || {
                loop{
                    trace!("Cache cleanup run..");
                    cache_cleanup.cleanup();
                    thread::sleep(Duration::from_secs(60));
                }
            });
        }

        rouille::start_server_with_pool(listen_str, Some(config.threads), move |request| {
            handle_connection(&connection_new, request, config.clone(), registry.clone(), cache.clone())
        });
    }
}

fn get_status<A>(connection_new: &A) -> Result<Status, Box<dyn std::error::Error>> where A: DbConnection {
    let version = env!("CARGO_PKG_VERSION");
    Ok(
        Status::new(
            1,
            Some(version.to_string()),
            "OK".to_string(),
            connection_new.get_station_count_working()?,
            connection_new.get_station_count_broken()?,
            connection_new.get_tag_count()?,
            connection_new.get_click_count_last_hour()?,
            connection_new.get_click_count_last_day()?,
            connection_new.get_language_count()?,
            connection_new.get_country_count()?,
        )
    )
}

fn send_file(path: &str, content_type: &'static str) -> ApiResponse {
    let file = File::open(path);
    match file {
        Ok(file) => ApiResponse::File(content_type.to_string(), file),
        _ => ApiResponse::NotFound,
    }
}

fn str_to_arr(string: &str) -> Vec<String> {
    let mut list: Vec<String> = vec![];
    let parts = string.split(",");
    for part in parts {
        let part_trimmed = part.trim().to_string();
        if part_trimmed != "" {
            list.push(part_trimmed);
        }
    }
    list
}

use std::fs::OpenOptions;
use std::io::prelude::*;

fn log_to_file(file_name: &str, line: &str) {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_name);

    match file {
        Ok(mut file) =>{
            if let Err(e) = writeln!(file, "{}", line) {
                error!("Couldn't write to file: {}", e);
            }
        },
        Err(err) => {
            error!("Could not open log file {}", err);
        }
    }
}

fn clean_url(original_url: &str) -> Result<&str, Box<dyn Error>>{
    let url_without_query: Vec<&str> = original_url.split("?").collect();
    if url_without_query.len() > 0 {
        let matches = url_without_query[0].matches("/");
        let filtered = match matches.count() {
            4 => {
                url_without_query[0].rsplitn(2, "/").collect::<Vec<&str>>()[1]
            },
            3 => {
                if url_without_query[0].contains("/stations/"){
                    url_without_query[0]
                }else{
                    url_without_query[0].rsplitn(2, "/").collect::<Vec<&str>>()[1]
                }
            },
            _ => url_without_query[0]
        };
        Ok(filtered)
    }else{
        return Err(Box::new(ApiError::InternalError(format!("Invalid url split result: {}", original_url))));
    }
}

fn handle_connection<A>(
    connection_new: &A,
    request: &rouille::Request,
    config: Config,
    registry: RegistryLinks,
    cache: cache::GenericCache,
) -> rouille::Response where A: DbConnection {
    let remote_ip: String = request.header("X-Forwarded-For").unwrap_or(&request.remote_addr().ip().to_string()).to_string();
    let referer: String = request.header("Referer").unwrap_or(&"-".to_string()).to_string();
    let user_agent: String = request.header("User-agent").unwrap_or(&"-".to_string()).to_string();

    let log_dir = config.log_dir.clone();
    let now = chrono::Utc::now().format("%d/%m/%Y:%H:%M:%S%.6f");
    let registry2 = registry.clone();
    let log_ok = |req: &Request, resp: &Response, elap: std::time::Duration| {
        let cleaned_url = clean_url(req.raw_url());
        match cleaned_url {
            Ok(cleaned_url) => {
                registry2.api_calls.with_label_values(&[req.method(), cleaned_url, &resp.status_code.to_string()]).inc();
            },
            Err(err) => {
                error!("Invalid url split result: {} {}", req.raw_url(), err);
            }
        }

        let line = format!(r#"{} {},{:09} - [{}] "{} {}" {} {} "{}" "{}""#, remote_ip, elap.as_secs(), elap.subsec_nanos(), now, req.method(), req.raw_url(), resp.status_code, 0, referer, user_agent);
        debug!("{}", line);
        let log_file = format!("{}/access.log",log_dir);
        log_to_file(&log_file, &line);
    };
    let log_err = |req: &Request, elap: std::time::Duration| {
        let cleaned_url = clean_url(req.raw_url());
        match cleaned_url {
            Ok(cleaned_url) => {
                registry2.api_calls.with_label_values(&[req.method(), cleaned_url, "500"]).inc();
            },
            Err(err) => {
                error!("Invalid url split result: {} {}", req.raw_url(), err);
            }
        }

        let line = format!(r#"{} {},{:09} - [{}] "{} {}" {}"#, remote_ip, elap.as_secs(), elap.subsec_nanos(), now, req.method(), req.raw_url(), 500);
        error!("{}", line);
        let log_file = format!("{}/access.log", log_dir);
        log_to_file(&log_file, &line);
    };
    rouille::log_custom(request, log_ok, log_err, || {
        let timer = registry.timer.with_label_values(&[request.method()]).start_timer();
        let result = handle_cached_connection(connection_new, request, config, registry, cache);
        let r = match result {
            Ok(response) => add_cors(response),
            Err(err) => {
                let err_str = err.to_string();
                error!("{}", err_str);
                add_cors(rouille::Response::text(err_str).with_status_code(500))
            } 
        };
        timer.observe_duration();
        r
    })
}

fn handle_cached_connection<A>(
    connection_new: &A,
    request: &rouille::Request,
    config: Config,
    registry: RegistryLinks,
    mut cache: cache::GenericCache,
) -> Result<rouille::Response, Box<dyn std::error::Error>> where A: DbConnection {
    if request.method() == "OPTIONS" {
        return Ok(rouille::Response::empty_204());
    }
    if request.method() != "POST" && request.method() != "GET" {
        return Ok(rouille::Response::empty_404());
    }

    let header_host = request.header("X-Forwarded-Host").or(request.header("Host"));
    let base_url = match header_host {
        Some(header_host) => format!("http://{host}", host = header_host),
        None => config.server_url.clone(),
    };
    trace!("header_host: {:?}", header_host);
    trace!("base_url: {:?}", base_url);
    let content_type_raw: &str = request.header("Content-Type").unwrap_or("nothing");
    let content_type_arr: Vec<&str> = content_type_raw.split(";").collect();
    if content_type_arr.len() == 0{
        return Ok(rouille::Response::empty_400());
    }
    let content_type = content_type_arr[0].trim();

    let remote_ip: String = request.header("X-Forwarded-For").unwrap_or(&request.remote_addr().ip().to_string()).to_string();

    let ppp = RequestParameters::new(&request);

    let allparams = AllParameters {
        url: request.raw_url().to_string(),
        param_uuids: str_to_arr(&ppp.get_string("uuids").unwrap_or(String::new())).iter().map(|item|item.to_lowercase()).collect(),
        param_tags: ppp.get_string("tags"),
        param_homepage: ppp.get_string("homepage"),
        param_favicon: ppp.get_string("favicon"),
    
        param_last_changeuuid: ppp.get_string("lastchangeuuid"),
        param_last_checkuuid: ppp.get_string("lastcheckuuid"),
        param_last_clickuuid: ppp.get_string("lastclickuuid"),
    
        param_name: ppp.get_string("name"),
        param_name_exact: ppp.get_bool("nameExact", false),
        param_country: ppp.get_string("country"),
        param_country_exact: ppp.get_bool("countryExact", false),
        param_countrycode: ppp.get_string("countrycode"),
        param_state: ppp.get_string("state"),
        param_state_exact: ppp.get_bool("stateExact", false),
        param_language: ppp.get_string("language"),
        param_language_codes: ppp.get_string("languagecodes"),
        param_language_exact: ppp.get_bool("languageExact", false),
        param_tag: ppp.get_string("tag"),
        param_tag_exact: ppp.get_bool("tagExact", false),
        param_tag_list: str_to_arr(&ppp.get_string("tagList").unwrap_or(String::new())),
        param_codec: ppp.get_string("codec"),
    
        param_bitrate_min: ppp.get_number("bitrateMin", 0),
        param_bitrate_max: ppp.get_number("bitrateMax", 1000000),
        param_order: ppp.get_string("order").unwrap_or(String::from("name")),
        param_reverse: ppp.get_bool("reverse", false),
        param_hidebroken: ppp.get_bool("hidebroken", false),
        param_has_geo_info: ppp.get_bool_opt("has_geo_info")?,
        param_has_extended_info: ppp.get_bool_opt("has_extended_info")?,
        param_is_https: ppp.get_bool_opt("is_https")?,
        
        param_offset: ppp.get_number("offset", 0),
        param_limit: ppp.get_number("limit", 999999),
    
        param_seconds: ppp.get_number("seconds", 0),
        param_url: ppp.get_string("url"),
        param_geo_lat: ppp.get_double("geo_lat", None),
        param_geo_long: ppp.get_double("geo_long", None),
    };

    let key = allparams.to_string()?;
    let cached_item = cache.get(&key);
    let mut is_text = false;
    let result: rouille::Response = match cached_item {
        Some(cached_item) => {
            registry.cache_hits.inc();
            is_text = true;
            rouille::Response::text(cached_item)
        },
        None => {
            registry.cache_misses.inc();
            let (do_cache, response) = do_api_calls(allparams, connection_new, config, registry, base_url, content_type, remote_ip)?;

            match response {
                ApiResponse::Text(text) => {
                    is_text = true;
                    if do_cache {
                        cache.set(&key, &text);
                        rouille::Response::text(text)
                    }else{
                        rouille::Response::text(text).with_no_cache()
                    }
                },
                ApiResponse::File(content_type, file) => {
                    rouille::Response::from_file(content_type, file)
                },
                ApiResponse::NotFound => {
                    rouille::Response::empty_404()
                },
                ApiResponse::UnknownContentType => {
                    rouille::Response::empty_406()
                },
                ApiResponse::ServerError(msg) => {
                    rouille::Response::text(msg).with_status_code(500)
                },
                ApiResponse::Locked(msg) => {
                    rouille::Response::text(msg).with_status_code(423)
                },
                /*
                ApiResponse::ParameterError(msg) => {
                    rouille::Response::text(msg).with_status_code(400)
                },
                */
            }
        }
    };

    if is_text {
        let url_path = request.url();
        let url_parts: Vec<&str> = url_path.split('/').collect();
        let response = if url_parts.len() > 1 {
            let output_content_type_short = url_parts[1];
            trace!("Parsed output content type: '{}'",output_content_type_short);
            match output_content_type_short {
                "html" => result.with_unique_header("Content-Type", "text/html"),
                "" => result.with_unique_header("Content-Type", "text/html"),
                "json" => result.with_unique_header("Content-Type", "application/json"),
                "xml" => result.with_unique_header("Content-Type", "text/xml"),
                "m3u" => result.with_unique_header("Content-Type", "audio/mpegurl").with_unique_header("Content-Disposition", r#"inline; filename="playlist.m3u""#),
                "pls" => result.with_unique_header("Content-Type", "audio/x-scpls").with_unique_header("Content-Disposition", r#"inline; filename="playlist.pls""#),
                "xspf" => result.with_unique_header("Content-Type", "application/xspf+xml").with_unique_header("Content-Disposition", r#"inline; filename="playlist.xspf""#),
                "ttl" => result.with_unique_header("Content-Type", "text/turtle"),
                _ => result,
            }
        }else{
            result
        };

        Ok(response)
    }else{
        Ok(result)
    }
}

fn do_api_calls<A>(all_params: AllParameters,
    connection_new: &A,
    config: Config,
    registry: RegistryLinks,
    base_url: String,
    content_type: &str,
    remote_ip: String,
) -> Result<(bool, ApiResponse), Box<dyn Error>> where A: DbConnection {
    use percent_encoding::{percent_decode_str};
    trace!("content_type: {}", content_type);
    let parts : Vec<&str> = all_params.url.split('?').collect();
    let items : Vec<String> = parts[0].split('/').map(|item| {
        let x = percent_decode_str(item);
        let y = x.decode_utf8_lossy();
        y.into_owned()
    }).collect();

    if items.len() == 2 {
        let file_name: &str = &items[1];
        match file_name {
            "metrics" => {
                if config.prometheus_exporter {
                    Ok((false, prometheus_exporter::render(connection_new, config.broken_stations_never_working_timeout.as_secs(), config.broken_stations_timeout.as_secs(), registry)?))
                }else{
                    Ok((true, ApiResponse::Locked("Exporter not enabled!".to_string())))
                }
            },
            "favicon.ico" => Ok((true,send_file(&format!("{}/{}",config.static_files_dir,"favicon.ico"), "image/png"))),
            "robots.txt" => Ok((true,send_file(&format!("{}/{}",config.static_files_dir,"robots.txt"), "text/plain"))),
            "main.css" => Ok((true,send_file(&format!("{}/{}",config.static_files_dir,"main.css"),"text/css"))),
            "" => {
                let mut handlebars = Handlebars::new();
                let y = handlebars.register_template_file("docs.hbs", &format!("{}/{}",config.static_files_dir,"docs.hbs"));
                if y.is_ok() {
                    let pkg_version = env!("CARGO_PKG_VERSION");
                    let mut data = Map::new();
                    data.insert(String::from("API_SERVER"), to_json(base_url));
                    data.insert(String::from("SERVER_VERSION"), to_json(format!("{version}",version = pkg_version)));
                    let rendered = handlebars.render("docs.hbs", &data)?;
                    Ok((true, ApiResponse::Text(rendered)))
                }else{
                    error!("unable register template file: docs.hbs");
                    Ok((false, ApiResponse::ServerError("unable to render docs".to_string())))
                }
            }
            _ => Ok((false, ApiResponse::NotFound)),
        }
    } else if items.len() == 3 {
        let format:&str = &items[1];
        let command:&str = &items[2];
        let filter : Option<String> = None;

        match command {
            "languages" => Ok((true,ApiLanguage::get_response(connection_new.get_extra("LanguageCache", "LanguageName", filter, all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format)?)),
            "countries" => Ok((true,encode_extra(connection_new.get_1_n("Country", filter, all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "country")?)),
            "countrycodes" => Ok((true,encode_extra(connection_new.get_1_n("CountryCode", filter, all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "countrycode")?)),
            "states" => Ok((true,encode_states(connection_new.get_states(None, filter, all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format)?)),
            "codecs" => Ok((true,encode_extra(connection_new.get_1_n("Codec", filter, all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "codec")?)),
            "tags" => Ok((true,encode_extra(connection_new.get_extra("TagCache", "TagName", filter, all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "tag")?)),
            "stations" => Ok((true,Station::get_response(connection_new.get_stations_by_all(&all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?.drain(..).map(|x|x.into()).collect(), format)?)),
            "servers" => Ok((true,dns_resolve(format)?)),
            "stats" => Ok((true,encode_status(get_status(connection_new)?, format, &config.static_files_dir))),
            "checks" => Ok((true,StationCheck::get_response(connection_new.get_checks(None, all_params.param_last_checkuuid, all_params.param_seconds, false, all_params.param_limit)?.drain(..).map(|x|x.into()).collect(),format)?)),
            "clicks" => Ok((true,StationClick::get_response(connection_new.get_clicks(None, all_params.param_last_clickuuid, all_params.param_seconds)?.drain(..).map(|x|x.into()).collect(),format)?)),
            "checksteps" => Ok((true,StationCheckStep::get_response(connection_new.select_station_check_steps_by_stations(&all_params.param_uuids)?.drain(..).map(|x|x.into()).collect(), format)?)),
            "add" => Ok((false,StationAddResult::from(connection_new.add_station_opt(all_params.param_name, all_params.param_url, all_params.param_homepage, all_params.param_favicon, all_params.param_countrycode, all_params.param_state, all_params.param_language, all_params.param_language_codes, all_params.param_tags, all_params.param_geo_lat, all_params.param_geo_long)).get_response(format)?)),
            "config" => Ok((true,ApiConfig::get_response(config.into(),format)?)),
            "streamingservers" => Ok((true,ApiStreamingServer::get_response(connection_new.get_streaming_servers(&all_params.param_order, all_params.param_reverse, all_params.param_offset, all_params.param_limit)?,format)?)),
            _ => Ok((true,ApiResponse::NotFound)),
        }
    } else if items.len() == 4 {
        let format:&str = &items[1];
        let command:&str = &items[2];
        let parameter:&str = &items[3];

        // None => connection_new.get_1_n("Country", filter, param_order, param_reverse, param_hidebroken)?, format, "country")?,
        match command {
            "languages" => Ok((true,ApiLanguage::get_response(connection_new.get_extra("LanguageCache", "LanguageName", Some(String::from(parameter)), all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format)?)),
            "countries" => Ok((true,encode_extra(connection_new.get_1_n("Country", Some(String::from(parameter)), all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "country")?)),
            "countrycodes" => Ok((true,encode_extra(connection_new.get_1_n("CountryCode", Some(String::from(parameter)), all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "countrycode")?)),
            "codecs" => Ok((true,encode_extra(connection_new.get_1_n("Codec", Some(String::from(parameter)), all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "codec")?)),
            "tags" => Ok((true,encode_extra(connection_new.get_extra("TagCache", "TagName", Some(String::from(parameter)), all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format, "tag")?)),
            "states" => Ok((true,encode_states(connection_new.get_states(None, Some(String::from(parameter)), all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format)?)),
            "vote" => Ok((false,encode_message(connection_new.vote_for_station(&remote_ip, get_only_first_item(connection_new.get_station_by_uuid(parameter)?)), format)?)),
            "url" => Ok((false,encode_station_url(connection_new, get_only_first_item(connection_new.get_station_by_uuid(parameter)?), &remote_ip, format, config.click_valid_timeout.as_secs(),registry)?)),
            "stations" => {
                match parameter {
                    "topvote" => Ok((true,Station::get_response(connection_new.get_stations_topvote(all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "topclick" => Ok((true,Station::get_response(connection_new.get_stations_topclick(all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "lastclick" => Ok((true,Station::get_response(connection_new.get_stations_lastclick(all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "lastchange" => Ok((true,Station::get_response(connection_new.get_stations_lastchange(all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "broken" => Ok((true,Station::get_response(connection_new.get_stations_broken(all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "changed" => Ok((true,encode_changes(connection_new.get_changes(None, all_params.param_last_changeuuid, all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "byurl" => Ok((true,Station::get_response(connection_new.get_stations_by_column_multiple("Url", all_params.param_url,true,&all_params.param_order,all_params.param_reverse,
                        all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "byserveruuid" => Ok((true,Station::get_response(connection_new.get_stations_by_server_uuids(all_params.param_uuids, &all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    //"byserverurl" => Ok((true,Station::get_response(connection_new.get_stations_by_uuid(all_params.param_uuids)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "byuuid" => Ok((true,Station::get_response(connection_new.get_stations_by_uuid(all_params.param_uuids)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "search" => Ok((true,Station::get_response(connection_new.get_stations_advanced(all_params.param_name, all_params.param_name_exact, all_params.param_country,
                        all_params.param_country_exact, all_params.param_countrycode, all_params.param_state, all_params.param_state_exact, all_params.param_language, all_params.param_language_exact, all_params.param_tag,
                        all_params.param_tag_exact, all_params.param_tag_list, all_params.param_codec, all_params.param_bitrate_min, all_params.param_bitrate_max, all_params.param_has_geo_info, all_params.param_has_extended_info, all_params.param_is_https, &all_params.param_order,all_params.param_reverse,
                        all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    _ => Ok((true,ApiResponse::NotFound)),
                }
            },
            "streamingservers" => {
                match parameter {
                    "byserveruuid" => Ok((true,ApiStreamingServer::get_response(connection_new.get_streaming_servers_by_uuids(all_params.param_uuids, &all_params.param_order, all_params.param_reverse, all_params.param_offset, all_params.param_limit)?,format)?)),
                    "bystationuuid" => Ok((true,ApiStreamingServer::get_response(connection_new.get_streaming_servers_by_station_uuids(all_params.param_uuids, &all_params.param_order, all_params.param_reverse, all_params.param_offset, all_params.param_limit)?,format)?)),
                    _ => Ok((true,ApiResponse::NotFound)),
                }
            },
            "checks" => Ok((true,StationCheck::get_response(connection_new.get_checks(Some(parameter.to_string()), all_params.param_last_checkuuid, all_params.param_seconds, true, all_params.param_limit)?.drain(..).map(|x|x.into()).collect(), format)?)),
            "clicks" => Ok((true,StationClick::get_response(connection_new.get_clicks(Some(parameter.to_string()), all_params.param_last_clickuuid, all_params.param_seconds)?.drain(..).map(|x|x.into()).collect(), format)?)),
            _ => Ok((true,ApiResponse::NotFound)),
        }
    } else if items.len() == 5 {
        let format:&str = &items[1];
        let command:&str = &items[2];
        let parameter:&str = &items[3];
        let search:&str = &items[4];
        if format == "v2" {
            // deprecated
            let format = command;
            let command = parameter;
            match command {
                "url" => Ok((false,encode_station_url(connection_new, get_only_first_item(connection_new.get_station_by_uuid(search)?), &remote_ip, format, config.click_valid_timeout.as_secs(), registry)?)),
                _ => Ok((false,ApiResponse::NotFound)),
            }
        }else{
            match command {
                "states" => Ok((true,encode_states(connection_new.get_states(Some(String::from(parameter)), Some(String::from(search)), all_params.param_order, all_params.param_reverse, all_params.param_hidebroken, all_params.param_offset, all_params.param_limit)?, format)?)),
                
                "stations" => {
                    match parameter {
                        "topvote" => Ok((true,Station::get_response(connection_new.get_stations_topvote(all_params.param_hidebroken, all_params.param_offset, search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "topclick" => Ok((true,Station::get_response(connection_new.get_stations_topclick(all_params.param_hidebroken, all_params.param_offset, search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "lastclick" => Ok((true,Station::get_response(connection_new.get_stations_lastclick(all_params.param_hidebroken, all_params.param_offset, search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "lastchange" => Ok((true,Station::get_response(connection_new.get_stations_lastchange(all_params.param_hidebroken, all_params.param_offset, search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "broken" => Ok((true,Station::get_response(connection_new.get_stations_broken(all_params.param_offset, search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "byname" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Name", search.to_string(),false,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bynameexact" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Name", search.to_string(),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycodec" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Codec", search.to_string(),false,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycodecexact" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Codec", search.to_string(),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycountry" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Country", search.to_string(),false,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycountryexact" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Country", search.to_string(),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycountrycodeexact" => Ok((true,Station::get_response(connection_new.get_stations_by_column("CountryCode", search.to_string(),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bystate" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Subcountry", search.to_string(),false,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bystateexact" => Ok((true,Station::get_response(connection_new.get_stations_by_column("Subcountry", search.to_string(),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bytag" => Ok((true,Station::get_response(connection_new.get_stations_by_column_multiple("Tags", Some(search.to_string()),false,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bytagexact" => Ok((true,Station::get_response(connection_new.get_stations_by_column_multiple("Tags", Some(search.to_string()),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bylanguage" => Ok((true,Station::get_response(connection_new.get_stations_by_column_multiple("Language", Some(search.to_string()),false,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bylanguageexact" => Ok((true,Station::get_response(connection_new.get_stations_by_column_multiple("Language", Some(search.to_string()),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "byuuid" => Ok((true,Station::get_response(connection_new.get_stations_by_column("StationUuid", search.to_string(),true,&all_params.param_order,all_params.param_reverse,all_params.param_hidebroken,all_params.param_offset,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "changed" => Ok((true,encode_changes(connection_new.get_changes(Some(search.to_string()),all_params.param_last_changeuuid,all_params.param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        _ => Ok((true,ApiResponse::NotFound)),
                    }
                },
                _ => Ok((true,ApiResponse::NotFound)),
            }
        }
    } else {
        Ok((true,ApiResponse::NotFound))
    }
}
