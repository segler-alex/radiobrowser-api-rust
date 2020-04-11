extern crate rouille;

extern crate serde;
extern crate serde_json;
extern crate dns_lookup;

pub mod data;
mod parameters;
mod prometheus_exporter;

use std::error::Error;

use self::parameters::RequestParameters;

use crate::api::data::ResultMessage;
use crate::api::data::StationCachedInfo;
use crate::api::data::StationHistoryCurrent;
use crate::api::data::Station;
use crate::api::data::StationCheck;
use crate::api::data::StationAddResult;
use crate::api::data::Status;
use crate::api::data::StationClick;
use crate::api::data::ApiConfig;
use crate::db::DbConnection;
use crate::db::models::ExtraInfo;
use crate::db::models::State;
use crate::db::models::StationItem;
use crate::api::rouille::Response;
use crate::api::rouille::Request;
use std;
use self::dns_lookup::lookup_host;
use self::dns_lookup::lookup_addr;

use crate::config::Config;

use std::fs::File;
use self::serde_json::value::{Map};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

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
        .with_unique_header("Access-Control-Allow-Headers", "origin, x-requested-with, content-type")
        .with_unique_header("Access-Control-Allow-Methods", "GET,POST")
}

fn get_only_first_item(mut stations: Vec<StationItem>) -> Option<StationItem>{
    if stations.len() == 1 {
        Some(stations.pop().unwrap())
    } else {
        None
    }
}

fn dns_resolve(format : &str) -> Result<rouille::Response, Box<dyn Error>> {
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
        "json" => {
            let j = serde_json::to_string(&list)?;
            Ok(rouille::Response::text(j)
                .with_no_cache()
                .with_unique_header("Content-Type","application/json"))
        },
        _ => Ok(rouille::Response::empty_404())
    }
}

fn encode_changes(list : Vec<StationHistoryCurrent>, format : &str) -> Result<rouille::Response, Box<dyn Error>> {
    Ok(match format {
        "json" => {
            let j = serde_json::to_string(&list)?;
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = StationHistoryCurrent::serialize_changes_list(list)?;
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    })
}

fn encode_message(status: Result<String, Box<dyn Error>>, format : &str) -> Result<rouille::Response, Box<dyn Error>> {
    Ok(match format {
        "json" => {
            match status {
                Ok(message) => rouille::Response::text(serde_json::to_string(&ResultMessage::new(true,message))?),
                Err(err) => rouille::Response::text(serde_json::to_string(&ResultMessage::new(false,err.to_string()))?),
            }.with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            match status {
                Ok(message) => rouille::Response::text(ResultMessage::new(true,message).serialize_xml()?),
                Err(err) => rouille::Response::text(ResultMessage::new(false,err.to_string()).serialize_xml()?),
            }.with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    })
}

fn encode_station_url<A>(connection_new: &A, station: Option<StationItem>, ip: &str, format : &str, seconds: u64, counter_clicks: Arc<AtomicUsize>) -> Result<rouille::Response, Box<dyn Error>> where A: DbConnection {
    Ok(match station {
        Some(station) => {
            counter_clicks.fetch_add(1,Ordering::Relaxed);
            let _ = connection_new.increase_clicks(&ip, &station, seconds);
            let station = station.into();
            match format {
                "json" => {
                    let s = Station::extract_cached_info(station, "retrieved station url");
                    let j = serde_json::to_string(&s)?;
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                },
                "xml" => {
                    let s = Station::extract_cached_info(station, "retrieved station url");
                    let j = StationCachedInfo::serialize_cached_info(s)?;
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
                },
                "m3u" => {
                    let list = vec![station];
                    let j = Station::serialize_to_m3u(list, true);
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","audio/mpegurl").with_unique_header("Content-Disposition", r#"inline; filename="playlist.m3u""#)
                },
                "pls" => {
                    let list = vec![station];
                    let j = Station::serialize_to_pls(list, true);
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","audio/x-scpls").with_unique_header("Content-Disposition", r#"inline; filename="playlist.pls""#)
                },
                _ => rouille::Response::empty_406()
            }
        },
        _ => rouille::Response::empty_404()
    })
}

fn encode_states(list : Vec<State>, format : &str) -> Result<rouille::Response, Box<dyn Error>> {
    Ok(match format {
        "json" => {
            let j = serde_json::to_string(&list)?;
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = State::serialize_state_list(list)?;
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    })
}

fn encode_extra(list : Vec<ExtraInfo>, format : &str, tag_name: &str) -> Result<rouille::Response, Box<dyn Error>> {
    Ok(match format {
        "json" => {
            let j = serde_json::to_string(&list)?;
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = ExtraInfo::serialize_extra_list(list, tag_name)?;
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    })
}

fn encode_status(status: Status, format : &str, static_dir: &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&status);
            match j {
                Ok(j) => rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json"),
                _ => rouille::Response::text("").with_status_code(500)
            }
        },
        "xml" => {
            let j = status.serialize_xml();
            match j {
                Ok(j) => rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml"),
                _ => rouille::Response::text("").with_status_code(500)
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
                    Ok(rendered) => rouille::Response::html(rendered).with_no_cache(),
                    _ => rouille::Response::text("").with_status_code(500)
                }
            }else{
                error!("unable register template file: stats.hbs");
                rouille::Response::text("").with_status_code(500)
            }
        },
        _ => rouille::Response::empty_406()
    }
}

pub fn start<A: 'static +  std::clone::Clone>(
    connection_new: A,
    config: Config,
) where A: DbConnection, A: std::marker::Send, A: std::marker::Sync {
    let listen_str = format!("{}:{}", config.listen_host, config.listen_port);
    info!("Listen on {} with {} threads", listen_str, config.threads);

    let counter_all = Arc::new(AtomicUsize::new(0));
    let counter_click = Arc::new(AtomicUsize::new(0));

    rouille::start_server_with_pool(listen_str, Some(config.threads), move |request| {
        let counter_all_2 = counter_all.clone();
        counter_all_2.fetch_add(1, Ordering::Relaxed);
        handle_connection(&connection_new, request, config.clone(), counter_all_2, counter_click.clone())
    });
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

fn send_file(path: &str, content_type: &'static str) -> rouille::Response {
    let file = File::open(path);
    match file {
        Ok(file) => {add_cors(rouille::Response::from_file(content_type, file))},
        _ => add_cors(rouille::Response::empty_404())
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

fn handle_connection<A>(
    connection_new: &A,
    request: &rouille::Request,
    config: Config,
    counter_all: Arc<AtomicUsize>,
    counter_clicks: Arc<AtomicUsize>,
) -> rouille::Response where A: DbConnection {
    let remote_ip: String = request.header("X-Forwarded-For").unwrap_or(&request.remote_addr().ip().to_string()).to_string();
    let referer: String = request.header("Referer").unwrap_or(&"-".to_string()).to_string();
    let user_agent: String = request.header("User-agent").unwrap_or(&"-".to_string()).to_string();

    let log_dir = config.log_dir.clone();
    let now = chrono::Utc::now().format("%d/%m/%Y:%H:%M:%S%.6f");
    let log_ok = |req: &Request, resp: &Response, _elap: std::time::Duration| {
        let line = format!(r#"{} - - [{}] "{} {}" {} {} "{}" "{}""#, remote_ip, now, req.method(), req.raw_url(), resp.status_code, 0, referer, user_agent);
        debug!("{}", line);
        let log_file = format!("{}/access.log",log_dir);
        log_to_file(&log_file, &line);
    };
    let log_err = |req: &Request, _elap: std::time::Duration| {
        let line = format!("{} {} Handler panicked: {} {}", remote_ip, now, req.method(), req.raw_url());
        debug!("{}", line);
        let log_file = format!("{}/error.log", log_dir);
        log_to_file(&log_file, &line);
    };
    rouille::log_custom(request, log_ok, log_err, || {
        let result = handle_connection_internal(connection_new, request, config, counter_all, counter_clicks);
        match result {
            Ok(response) => response,
            Err(err) => rouille::Response::text(err.to_string()).with_status_code(500),
        }
    })
}

fn handle_connection_internal<A>(
    connection_new: &A,
    request: &rouille::Request,
    config: Config,
    counter_all: Arc<AtomicUsize>,
    counter_clicks: Arc<AtomicUsize>,
) -> Result<rouille::Response, Box<dyn std::error::Error>> where A: DbConnection {
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
    
    let param_tags: Option<String> = ppp.get_string("tags");
    let param_homepage: Option<String> = ppp.get_string("homepage");
    let param_favicon: Option<String> = ppp.get_string("favicon");

    let param_last_changeuuid: Option<String> = ppp.get_string("lastchangeuuid");
    let param_last_checkuuid: Option<String> = ppp.get_string("lastcheckuuid");
    let param_last_clickuuid: Option<String> = ppp.get_string("lastclickuuid");

    let param_name: Option<String> = ppp.get_string("name");
    let param_name_exact: bool = ppp.get_bool("nameExact", false);
    let param_country: Option<String> = ppp.get_string("country");
    let param_country_exact: bool = ppp.get_bool("countryExact", false);
    let param_countrycode: Option<String> = ppp.get_string("countrycode");
    let param_state: Option<String> = ppp.get_string("state");
    let param_state_exact: bool = ppp.get_bool("stateExact", false);
    let param_language: Option<String> = ppp.get_string("language");
    let param_language_exact: bool = ppp.get_bool("languageExact", false);
    let param_tag: Option<String> = ppp.get_string("tag");
    let param_tag_exact: bool = ppp.get_bool("tagExact", false);
    let param_tag_list: Vec<String> = str_to_arr(&ppp.get_string("tagList").unwrap_or(String::new()));
    let param_codec: Option<String> = ppp.get_string("codec");

    let param_bitrate_min : u32 = ppp.get_number("bitrateMin", 0);
    let param_bitrate_max : u32 = ppp.get_number("bitrateMax", 1000000);
    let param_order : String = ppp.get_string("order").unwrap_or(String::from("name"));
    let param_reverse : bool = ppp.get_bool("reverse", false);
    let param_hidebroken : bool = ppp.get_bool("hidebroken", false);
    let param_offset : u32 = ppp.get_number("offset", 0);
    let param_limit : u32 = ppp.get_number("limit", 999999);

    let param_seconds: u32 = ppp.get_number("seconds", 0);
    let param_url: Option<String> = ppp.get_string("url");

    use percent_encoding::{percent_decode_str};
    trace!("content_type: {}", content_type);
    let parts : Vec<&str> = request.raw_url().split('?').collect();
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
                    Ok(prometheus_exporter::render(connection_new, &config.prometheus_exporter_prefix, config.broken_stations_never_working_timeout.as_secs(), config.broken_stations_timeout.as_secs(), counter_all, counter_clicks)?)
                }else{
                    Ok(rouille::Response::text("Exporter not enabled!").with_status_code(423))
                }
            },
            "favicon.ico" => Ok(send_file(&format!("{}/{}",config.static_files_dir,"favicon.ico"), "image/png")),
            "robots.txt" => Ok(send_file(&format!("{}/{}",config.static_files_dir,"robots.txt"), "text/plain")),
            "main.css" => Ok(send_file(&format!("{}/{}",config.static_files_dir,"main.css"),"text/css")),
            "" => {
                let mut handlebars = Handlebars::new();
                let y = handlebars.register_template_file("docs.hbs", &format!("{}/{}",config.static_files_dir,"docs.hbs"));
                if y.is_ok() {
                    let pkg_version = env!("CARGO_PKG_VERSION");
                    let mut data = Map::new();
                    data.insert(String::from("API_SERVER"), to_json(base_url));
                    data.insert(String::from("SERVER_VERSION"), to_json(format!("{version}",version = pkg_version)));
                    let rendered = handlebars.render("docs.hbs", &data)?;
                    Ok(rouille::Response::html(rendered).with_no_cache())
                }else{
                    error!("unable register template file: docs.hbs");
                    Ok(rouille::Response::text("").with_status_code(500))
                }
            }
            _ => Ok(rouille::Response::empty_404()),
        }
    } else if items.len() == 3 {
        let format:&str = &items[1];
        let command:&str = &items[2];
        let filter : Option<String> = None;

        match command {
            "languages" => Ok(add_cors(encode_extra(connection_new.get_extra("LanguageCache", "LanguageName", filter, param_order, param_reverse, param_hidebroken)?, format, "language")?)),
            "countries" => Ok(add_cors(encode_extra(connection_new.get_1_n("Country", filter, param_order, param_reverse, param_hidebroken)?, format, "country")?)),
            "countrycodes" => Ok(add_cors(encode_extra(connection_new.get_1_n("CountryCode", filter, param_order, param_reverse, param_hidebroken)?, format, "countrycode")?)),
            "states" => Ok(add_cors(encode_states(connection_new.get_states(None, filter, param_order, param_reverse, param_hidebroken)?, format)?)),
            "codecs" => Ok(add_cors(encode_extra(connection_new.get_1_n("Codec", filter, param_order, param_reverse, param_hidebroken)?, format, "codec")?)),
            "tags" => Ok(add_cors(encode_extra(connection_new.get_extra("TagCache", "TagName", filter, param_order, param_reverse, param_hidebroken)?, format, "tag")?)),
            "stations" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_all(&param_order, param_reverse, param_hidebroken, param_offset, param_limit)?.drain(..).map(|x|x.into()).collect(), format)?)),
            "servers" => Ok(add_cors(dns_resolve(format)?)),
            "stats" => Ok(add_cors(encode_status(get_status(connection_new)?, format, &config.static_files_dir))),
            "checks" => Ok(add_cors(StationCheck::get_response(connection_new.get_checks(None, param_last_checkuuid, param_seconds, false)?.drain(..).map(|x|x.into()).collect(),format)?)),
            "clicks" => Ok(add_cors(StationClick::get_response(connection_new.get_clicks(None, param_last_clickuuid, param_seconds)?.drain(..).map(|x|x.into()).collect(),format)?)),
            "add" => Ok(add_cors(StationAddResult::from(connection_new.add_station_opt(param_name, param_url, param_homepage, param_favicon, param_country, param_countrycode, param_state, param_language, param_tags)).get_response(format)?)),
            "config" => Ok(add_cors(ApiConfig::get_response(config.into(),format)?)),
            _ => Ok(rouille::Response::empty_404()),
        }
    } else if items.len() == 4 {
        let format:&str = &items[1];
        let command:&str = &items[2];
        let parameter:&str = &items[3];

        match command {
            "languages" => Ok(add_cors(encode_extra(connection_new.get_extra("LanguageCache", "LanguageName", Some(String::from(parameter)), param_order, param_reverse, param_hidebroken)?, format, "language")?)),
            "countries" => Ok(add_cors(encode_extra(connection_new.get_1_n("Country", Some(String::from(parameter)), param_order, param_reverse, param_hidebroken)?, format, "country")?)),
            "countrycodes" => Ok(add_cors(encode_extra(connection_new.get_1_n("CountryCode", Some(String::from(parameter)), param_order, param_reverse, param_hidebroken)?, format, "countrycode")?)),
            "codecs" => Ok(add_cors(encode_extra(connection_new.get_1_n("Codec", Some(String::from(parameter)), param_order, param_reverse, param_hidebroken)?, format, "codec")?)),
            "tags" => Ok(add_cors(encode_extra(connection_new.get_extra("TagCache", "TagName", Some(String::from(parameter)), param_order, param_reverse, param_hidebroken)?, format, "tag")?)),
            "states" => Ok(add_cors(encode_states(connection_new.get_states(None, Some(String::from(parameter)), param_order, param_reverse, param_hidebroken)?, format)?)),
            "vote" => Ok(add_cors(encode_message(connection_new.vote_for_station(&remote_ip, get_only_first_item(connection_new.get_station_by_uuid(parameter)?)), format)?)),
            "url" => Ok(add_cors(encode_station_url(connection_new, get_only_first_item(connection_new.get_station_by_uuid(parameter)?), &remote_ip, format, config.click_valid_timeout.as_secs(),counter_clicks)?)),
            "stations" => {
                match parameter {
                    "topvote" => Ok(add_cors(Station::get_response(connection_new.get_stations_topvote(999999)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "topclick" => Ok(add_cors(Station::get_response(connection_new.get_stations_topclick(999999)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "lastclick" => Ok(add_cors(Station::get_response(connection_new.get_stations_lastclick(999999)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "lastchange" => Ok(add_cors(Station::get_response(connection_new.get_stations_lastchange(999999)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "broken" => Ok(add_cors(Station::get_response(connection_new.get_stations_broken(999999)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "improvable" => Ok(add_cors(Station::get_response(connection_new.get_stations_improvable(999999)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "changed" => Ok(add_cors(encode_changes(connection_new.get_changes(None, param_last_changeuuid)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "byurl" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column_multiple("Url", param_url,true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    "search" => Ok(add_cors(Station::get_response(connection_new.get_stations_advanced(param_name, param_name_exact, param_country, param_country_exact, param_countrycode, param_state, param_state_exact, param_language, param_language_exact, param_tag, param_tag_exact, param_tag_list, param_codec, param_bitrate_min, param_bitrate_max, &param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                    _ => Ok(rouille::Response::empty_404()),
                }
            },
            "checks" => Ok(add_cors(StationCheck::get_response(connection_new.get_checks(Some(parameter.to_string()), param_last_checkuuid, param_seconds, true)?.drain(..).map(|x|x.into()).collect(), format)?)),
            "clicks" => Ok(add_cors(StationClick::get_response(connection_new.get_clicks(Some(parameter.to_string()), param_last_clickuuid, param_seconds)?.drain(..).map(|x|x.into()).collect(), format)?)),
            _ => Ok(rouille::Response::empty_404()),
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
                "url" => Ok(add_cors(encode_station_url(connection_new, get_only_first_item(connection_new.get_station_by_uuid(search)?), &remote_ip, format, config.click_valid_timeout.as_secs(), counter_clicks)?)),
                _ => Ok(rouille::Response::empty_404()),
            }
        }else{
            match command {
                "states" => Ok(add_cors(encode_states(connection_new.get_states(Some(String::from(parameter)), Some(String::from(search)), param_order, param_reverse, param_hidebroken)?, format)?)),
                
                "stations" => {
                    match parameter {
                        "topvote" => Ok(add_cors(Station::get_response(connection_new.get_stations_topvote(search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "topclick" => Ok(add_cors(Station::get_response(connection_new.get_stations_topclick(search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "lastclick" => Ok(add_cors(Station::get_response(connection_new.get_stations_lastclick(search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "lastchange" => Ok(add_cors(Station::get_response(connection_new.get_stations_lastchange(search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "broken" => Ok(add_cors(Station::get_response(connection_new.get_stations_broken(search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "improvable" => Ok(add_cors(Station::get_response(connection_new.get_stations_improvable(search.parse().unwrap_or(0))?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "byname" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Name", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bynameexact" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Name", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycodec" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Codec", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycodecexact" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Codec", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycountry" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Country", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycountryexact" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Country", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bycountrycodeexact" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("CountryCode", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bystate" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Subcountry", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bystateexact" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("Subcountry", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bytag" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column_multiple("Tags", Some(search.to_string()),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bytagexact" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column_multiple("Tags", Some(search.to_string()),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bylanguage" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column_multiple("Language", Some(search.to_string()),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "bylanguageexact" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column_multiple("Language", Some(search.to_string()),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "byuuid" => Ok(add_cors(Station::get_response(connection_new.get_stations_by_column("StationUuid", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        "changed" => Ok(add_cors(encode_changes(connection_new.get_changes(Some(search.to_string()),param_last_changeuuid)?.drain(..).map(|x| x.into()).collect(), format)?)),
                        _ => Ok(rouille::Response::empty_404()),
                    }
                },
                _ => Ok(rouille::Response::empty_404()),
            }
        }
    } else {
        Ok(rouille::Response::empty_404())
    }
}
