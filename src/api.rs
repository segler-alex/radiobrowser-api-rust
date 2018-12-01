extern crate rouille;

extern crate serde;
extern crate serde_json;
extern crate dns_lookup;

use std;
use db;
use self::dns_lookup::lookup_host;
use self::dns_lookup::lookup_addr;
use std::io::Read;

use url::form_urlencoded;

use std::fs::File;
use self::serde_json::value::{Map};

use handlebars::{
    to_json, Handlebars,
};

#[derive(Serialize, Deserialize)]
pub struct ServerEntry {
    ip: String,
    name: String
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    status: String
}

#[derive(Serialize, Deserialize)]
pub struct ResultMessage {
    ok: bool,
    message: String,
}

fn add_cors(result : rouille::Response) -> rouille::Response {
    result.with_unique_header("Access-Control-Allow-Origin", "*")
        .with_unique_header("Access-Control-Allow-Headers", "origin, x-requested-with, content-type")
        .with_unique_header("Access-Control-Allow-Methods", "GET,POST")
}

fn dns_resolve(format : &str) -> rouille::Response {
    let hostname = "api.radio-browser.info";
    let ips: Vec<std::net::IpAddr> = lookup_host(hostname).unwrap();
    let mut list: Vec<ServerEntry> = Vec::new();
    for ip in ips {
        let ip_str : String = format!("{}",ip);
        let name : String = lookup_addr(&ip).unwrap();
        let item = ServerEntry{ip: ip_str, name};
        list.push(item);
    }

    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j)
                .with_no_cache()
                .with_unique_header("Content-Type","application/json")
        },
        _ => rouille::Response::empty_404()
    }
}

fn get_changes(request: &rouille::Request, connection: &db::Connection, station_id : Option<String>) -> Vec<db::StationHistory>{
    let seconds: u32 = request.get_param("seconds").unwrap_or(String::from("0")).parse().unwrap_or(0);
    let stations = connection.get_changes(station_id, seconds);
    stations
}

fn get_1_n_with_parse(request: &rouille::Request, connection: &db::Connection, column: &str, filter_prev : Option<String>) -> Vec<db::Result1n>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let stations = connection.get_1_n(column, filter, order, reverse, hidebroken);
    stations
}

fn get_states_with_parse(request: &rouille::Request, connection: &db::Connection, country: Option<String>, filter_prev : Option<String>) -> Vec<db::State>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let stations = connection.get_states(country, filter, order, reverse, hidebroken);
    stations
}

fn get_tags_with_parse(request: &rouille::Request, connection: &db::Connection, filter_prev : Option<String>) -> Vec<db::ExtraInfo>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let tags = connection.get_extra("TagCache", "TagName", filter, order, reverse, hidebroken);
    tags
}

fn get_languages_with_parse(request: &rouille::Request, connection: &db::Connection, filter_prev : Option<String>) -> Vec<db::ExtraInfo>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let languages = connection.get_extra("LanguageCache", "LanguageName", filter, order, reverse, hidebroken);
    languages
}

/*fn encode_result1n_xml_single(entry: db::Result1n) -> String{
    encode_result1n_xml
}*/

pub fn serialize_result_message(result: ResultMessage) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
        xml.begin_elem("status")?;
            xml.attr_esc("ok", &result.ok.to_string())?;
            xml.attr_esc("message", &result.message)?;
        xml.end_elem()?;
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
}

fn encode_result1n(type_str: &str, list : Vec<db::Result1n>, format : &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = db::serialize_result1n_list(type_str, list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    }
}

fn encode_changes(list : Vec<db::StationHistory>, format : &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = db::serialize_changes_list(list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    }
}

fn encode_stations(list : Vec<db::Station>, format : &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = db::serialize_station_list(list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        "m3u" => {
            let j = db::serialize_to_m3u(list, false);
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","audio/mpegurl").with_unique_header("Content-Disposition", r#"inline; filename="playlist.m3u""#)
        },
        "pls" => {
            let j = db::serialize_to_pls(list, false);
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","audio/x-scpls").with_unique_header("Content-Disposition", r#"inline; filename="playlist.pls""#)
        },
        "xspf" => {
            let j = db::serialize_to_xspf(list).unwrap();
            rouille::Response::text(j).with_unique_header("Content-Type","application/xspf+xml").with_unique_header("Content-Disposition", r#"inline; filename="playlist.xspf""#)
        },
        "ttl" => {
            let j = db::serialize_to_ttl(list);
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/turtle")
        },
        _ => rouille::Response::empty_406()
    }
}

fn encode_message(status: Result<String,String>, format : &str) -> rouille::Response {
    match format {
        "json" => {
            match status {
                Ok(message) => rouille::Response::text(serde_json::to_string(&ResultMessage{ok:true,message:message}).unwrap()).with_no_cache().with_unique_header("Content-Type","application/json"),
                Err(message) => rouille::Response::text(serde_json::to_string(&ResultMessage{ok:false,message:message}).unwrap()).with_no_cache().with_unique_header("Content-Type","application/json"),
            }
        },
        "xml" => {
            match status {
                Ok(message) => rouille::Response::text(serialize_result_message(ResultMessage{ok:true,message:message}).unwrap()).with_no_cache().with_unique_header("Content-Type","text/xml"),
                Err(message) => rouille::Response::text(serialize_result_message(ResultMessage{ok:false,message:message}).unwrap()).with_no_cache().with_unique_header("Content-Type","text/xml"),
            }
        },
        _ => rouille::Response::empty_406()
    }
}

fn encode_station_url(connection: &db::Connection, station: Option<db::Station>, ip: &str, format : &str) -> rouille::Response {
    match station {
        Some(station) => {
            connection.increase_clicks(&ip, &station);

            match format {
                "json" => {
                    let s = db::extract_cached_info(station, "retrieved station url");
                    let j = serde_json::to_string(&s).unwrap();
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                },
                "xml" => {
                    let s = db::extract_cached_info(station, "retrieved station url");
                    let j = db::serialize_cached_info(s).unwrap();
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
                },
                "m3u" => {
                    let list = vec![station];
                    let j = db::serialize_to_m3u(list, true);
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","audio/mpegurl").with_unique_header("Content-Disposition", r#"inline; filename="playlist.m3u""#)
                },
                "pls" => {
                    let list = vec![station];
                    let j = db::serialize_to_pls(list, true);
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","audio/x-scpls").with_unique_header("Content-Disposition", r#"inline; filename="playlist.pls""#)
                },
                _ => rouille::Response::empty_406()
            }
        },
        _ => rouille::Response::empty_404()
    }
}

fn encode_states(list : Vec<db::State>, format : &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = db::serialize_state_list(list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    }
}

fn encode_extra(list : Vec<db::ExtraInfo>, format : &str, tag_name: &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = db::serialize_extra_list(list, tag_name).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    }
}

fn encode_checks(list: Vec<db::StationCheck>, format: &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = db::serialize_station_checks(list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    }
}

pub fn serialize_status(status: &Status) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    {
        xml.begin_elem("status")?;
        let s = status.status.clone();
            xml.attr_esc("value", &s)?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap())
}

fn encode_status(status: Status, format : &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&status);
            match j {
                Ok(j) => rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json"),
                _ => rouille::Response::text("").with_status_code(500)
            }
        },
        "xml" => {
            let j = serialize_status(&status);
            match j {
                Ok(j) => rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml"),
                _ => rouille::Response::text("").with_status_code(500)
            }
        },
        "html" => {
            let mut handlebars = Handlebars::new();
            let y = handlebars.register_template_file("template.html", "static/template.html");
            if y.is_ok(){
                let mut data = Map::new();
                data.insert(String::from("status"), to_json(status));
                let rendered = handlebars.render("template.hbs", &data);
                match rendered {
                    Ok(rendered) => rouille::Response::html(rendered).with_no_cache(),
                    _ => rouille::Response::text("").with_status_code(500)
                }
            }else{
                rouille::Response::text("").with_status_code(500)
            }
        },
        _ => rouille::Response::empty_406()
    }
}

pub fn run(connection: db::Connection, host : String, port : i32, threads : usize, server_name: &str) {
    let listen_str = format!("{}:{}", host, port);
    println!("Listen on {} with {} threads", listen_str, threads);
    let x : Option<usize> = Some(threads);
    let y = String::from(server_name);
    rouille::start_server_with_pool(listen_str, x, move |request| {
    //rouille::start_server(listen_str, move |request| {
        //rouille::log(&request, std::io::stdout(), || {
            handle_connection(&connection, request, &y)
        //})
    });
}

fn get_status() -> Status {
    Status{status: "OK".to_string()}
}

/*
fn send_file(path: &str) -> rouille::Response {
    let file = File::open(path);
    match file {
        Ok(file) => {add_cors(rouille::Response::from_file("text/html", file))},
        _ => add_cors(rouille::Response::empty_404())
    }
}
*/

fn send_image(path: &str) -> rouille::Response {
    let file = File::open(path);
    match file {
        Ok(file) => {add_cors(rouille::Response::from_file("image/png", file))},
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

fn handle_connection(connection: &db::Connection, request: &rouille::Request, server_name: &str) -> rouille::Response {
    if request.method() != "POST" && request.method() != "GET" {
        return rouille::Response::empty_404();
    }
    let header_host: &str = request.header("Host").unwrap_or(server_name);
    let content_type: &str = request.header("Content-Type").unwrap_or("nothing");



    let mut param_name: Option<String> = request.get_param("name");
    let mut param_name_exact: bool = request.get_param("nameExact").unwrap_or(String::from("false")).parse().unwrap_or(false);
    let mut param_country: Option<String> = request.get_param("country");
    let mut param_country_exact: bool = request.get_param("countryExact").unwrap_or(String::from("false")).parse().unwrap_or(false);
    let mut param_state: Option<String> = request.get_param("state");
    let mut param_state_exact: bool = request.get_param("stateExact").unwrap_or(String::from("false")).parse().unwrap_or(false);
    let mut param_language: Option<String> = request.get_param("language");
    let mut param_language_exact: bool = request.get_param("languageExact").unwrap_or(String::from("false")).parse().unwrap_or(false);
    let mut param_tag: Option<String> = request.get_param("tag");
    let mut param_tag_exact: bool = request.get_param("tagExact").unwrap_or(String::from("false")).parse().unwrap_or(false);
    let mut param_tag_list: Vec<String> = str_to_arr(&request.get_param("tagList").unwrap_or(String::from("")));

    let mut param_bitrate_min : u32 = request.get_param("bitrateMin").unwrap_or(String::from("0")).parse().unwrap_or(0);
    let mut param_bitrate_max : u32 = request.get_param("bitrateMax").unwrap_or(String::from("1000000")).parse().unwrap_or(1000000);
    let mut param_order : String = request.get_param("order").unwrap_or(String::from("value"));
    let mut param_reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let mut param_hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let mut param_offset : u32 = request.get_param("offset").unwrap_or(String::from("0")).parse().unwrap_or(0);
    let mut param_limit : u32 = request.get_param("limit").unwrap_or(String::from("999999")).parse().unwrap_or(999999);

    let mut param_seconds: u32 = request.get_param("seconds").unwrap_or(String::from("0")).parse().unwrap_or(0);
    let mut param_url: String = String::from("");

    let ip = request.remote_addr().ip().to_string();
    
    if request.method() == "POST" {
        match content_type {
            "application/x-www-form-urlencoded" => {
                let mut data = request.data().unwrap();
                let mut buf = Vec::new();
                match data.read_to_end(&mut buf) {
                    Ok(_) => {
                        let iter = form_urlencoded::parse(&buf);
                        for (key,val) in iter {
                            if key == "order" { param_order = val.into(); }
                            else if key == "name" { param_name = Some(val.into()); }
                            else if key == "nameExact" { param_name_exact = val.parse().unwrap_or(param_name_exact); }
                            else if key == "country" { param_country = Some(val.into()); }
                            else if key == "countryExact" { param_country_exact = val.parse().unwrap_or(param_country_exact); }
                            else if key == "state" { param_state = Some(val.into()); }
                            else if key == "stateExact" { param_state_exact = val.parse().unwrap_or(param_state_exact); }
                            else if key == "language" { param_language = Some(val.into()); }
                            else if key == "languageExact" { param_language_exact = val.parse().unwrap_or(param_language_exact); }
                            else if key == "tag" { param_tag = Some(val.into()); }
                            else if key == "tagExact" { param_tag_exact = val.parse().unwrap_or(param_tag_exact); }
                            else if key == "tagList" { 
                                let x: String = val.into();
                                param_tag_list = str_to_arr(&x);
                            }
                            else if key == "reverse" { param_reverse = val.parse().unwrap_or(param_reverse); }
                            else if key == "hidebroken" { param_hidebroken = val.parse().unwrap_or(param_hidebroken); }
                            else if key == "bitrateMin" { param_bitrate_min = val.parse().unwrap_or(param_bitrate_min); }
                            else if key == "bitrateMax" { param_bitrate_max = val.parse().unwrap_or(param_bitrate_max); }
                            else if key == "offset" { param_offset = val.parse().unwrap_or(param_offset); }
                            else if key == "limit" { param_limit = val.parse().unwrap_or(param_limit); }
                            else if key == "seconds" { param_seconds = val.parse().unwrap_or(param_seconds); }
                            else if key == "url" { param_url = val.into(); }
                        }
                    },
                    Err(err) => {
                        println!("err {}",err);
                    }
                }
            },
            "application/json" => {
                let mut data = request.data().unwrap();
                let mut buf = Vec::new();
                match data.read_to_end(&mut buf) {
                    Ok(_) => {
                        let v: self::serde_json::Value = serde_json::from_slice(&buf).unwrap();
                        // name
                        if v["name"].is_string() {
                            param_name = Some(v["name"].as_str().unwrap().to_string());
                        }
                        if v["nameExact"].is_boolean() {
                            param_name_exact = v["nameExact"].as_bool().unwrap();
                        }
                        if v["nameExact"].is_string() {
                            param_name_exact = v["nameExact"].as_str().unwrap().parse().unwrap_or(param_name_exact);
                        }
                        // country
                        if v["country"].is_string() {
                            param_country = Some(v["country"].as_str().unwrap().to_string());
                        }
                        if v["countryExact"].is_boolean() {
                            param_country_exact = v["countryExact"].as_bool().unwrap();
                        }
                        if v["countryExact"].is_string() {
                            param_country_exact = v["countryExact"].as_str().unwrap().parse().unwrap_or(param_country_exact);
                        }
                        // state
                        if v["state"].is_string() {
                            param_state = Some(v["state"].as_str().unwrap().to_string());
                        }
                        if v["stateExact"].is_boolean() {
                            param_state_exact = v["stateExact"].as_bool().unwrap();
                        }
                        if v["stateExact"].is_string() {
                            param_state_exact = v["stateExact"].as_str().unwrap().parse().unwrap_or(param_state_exact);
                        }
                        // language
                        if v["language"].is_string() {
                            param_language = Some(v["language"].as_str().unwrap().to_string());
                        }
                        if v["languageExact"].is_boolean() {
                            param_language_exact = v["languageExact"].as_bool().unwrap();
                        }
                        if v["nameExact"].is_string() {
                            param_language_exact = v["languageExact"].as_str().unwrap().parse().unwrap_or(param_language_exact);
                        }
                        // tag
                        if v["tag"].is_string() {
                            param_tag = Some(v["tag"].as_str().unwrap().to_string());
                        }
                        if v["tagExact"].is_boolean() {
                            param_tag_exact = v["tagExact"].as_bool().unwrap();
                        }
                        if v["tagExact"].is_string() {
                            param_tag_exact = v["tagExact"].as_str().unwrap().parse().unwrap_or(param_tag_exact);
                        }
                        if v["tagList"].is_array() {
                            let x = v["tagList"].as_array().unwrap();
                            param_tag_list = x.into_iter().map(|item| {
                                if item.is_string(){
                                    item.as_str().unwrap().trim().to_string()
                                }else{
                                    String::from("")
                                }
                            }).filter(|item| {
                                item != ""
                            }).collect();
                        }
                        if v["tagList"].is_string() {
                            param_tag_list = str_to_arr(v["tagList"].as_str().unwrap());
                        }
                        // other
                        if v["order"].is_string() {
                            param_order = v["order"].as_str().unwrap().to_string();
                        }
                        if v["url"].is_string() {
                            param_url = v["url"].as_str().unwrap().to_string();
                        }
                        if v["reverse"].is_string() {
                            param_reverse = v["reverse"].as_str().unwrap().parse().unwrap_or(param_reverse);
                        }
                        if v["reverse"].is_boolean() {
                            param_reverse = v["reverse"].as_bool().unwrap();
                        }
                        if v["hidebroken"].is_string() {
                            param_hidebroken = v["hidebroken"].as_str().unwrap() == "true";
                        }
                        if v["hidebroken"].is_boolean() {
                            param_hidebroken = v["hidebroken"].as_bool().unwrap();
                        }
                        param_offset = v["offset"].as_u64().unwrap_or(param_offset.into()) as u32;
                        param_limit = v["limit"].as_u64().unwrap_or(param_limit.into()) as u32;
                        param_bitrate_min = v["bitrateMin"].as_u64().unwrap_or(param_bitrate_min.into()) as u32;
                        param_bitrate_max = v["bitrateMax"].as_u64().unwrap_or(param_bitrate_max.into()) as u32;
                        param_seconds = v["seconds"].as_u64().unwrap_or(param_seconds.into()) as u32;
                    },
                    Err(err) => {
                        println!("err {}",err);
                    }
                }
            },
            _ => {
            }
        }
    }

    let parts : Vec<&str> = request.raw_url().split('?').collect();
    let items : Vec<&str> = parts[0].split('/').collect();
    if items.len() == 2 {
        let file_name = items[1];
        match file_name {
            "favicon.ico" => send_image("static/favicon.ico"),
            "" => {
                let mut handlebars = Handlebars::new();
                let y = handlebars.register_template_file("docs.hbs", "static/docs.hbs");
                if y.is_ok() {
                    let mut data = Map::new();
                    data.insert(String::from("API_SERVER"), to_json(format!("http://{name}",name = header_host)));
                    let rendered = handlebars.render("docs.hbs", &data);
                    match rendered {
                        Ok(rendered) => rouille::Response::html(rendered).with_no_cache(),
                        _ => rouille::Response::text("").with_status_code(500)
                    }
                }else{
                    rouille::Response::text("").with_status_code(500)
                }
            }
            _ => rouille::Response::empty_404(),
        }
    } else if items.len() == 3 {
        let format = items[1];
        let command = items[2];
        let filter : Option<String> = None;

        match command {
            "languages" => add_cors(encode_extra(get_languages_with_parse(&request, &connection, filter), format, "language")),
            "countries" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Country", filter), format)),
            "states" => add_cors(encode_states(get_states_with_parse(&request, &connection, None, filter), format)),
            "codecs" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Codec", filter), format)),
            "tags" => add_cors(encode_extra(get_tags_with_parse(&request, &connection, filter), format, "tag")),
            "stations" => add_cors(encode_stations(connection.get_stations_by_all(&param_order, param_reverse, param_hidebroken, param_offset, param_limit), format)),
            "servers" => add_cors(dns_resolve(format)),
            "status" => add_cors(encode_status(get_status(), format)),
            "checks" => add_cors(encode_checks(connection.get_checks(None, param_seconds),format)),
            _ => rouille::Response::empty_404()
        }
    } else if items.len() == 4 {
        let format = items[1];
        let command = items[2];
        let parameter = items[3];

        match command {
            "languages" => add_cors(encode_extra(get_languages_with_parse(&request, &connection, Some(String::from(parameter))), format, "language")),
            "countries" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Country", Some(String::from(parameter))), format)),
            "codecs" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Codec", Some(String::from(parameter))), format)),
            "tags" => add_cors(encode_extra(get_tags_with_parse(&request, &connection, Some(String::from(parameter))), format, "tag")),
            "states" => add_cors(encode_states(get_states_with_parse(&request, &connection, None, Some(String::from(parameter))), format)),
            "vote" => add_cors(encode_message(connection.vote_for_station(&ip, connection.get_station_by_id_or_uuid(parameter)), format)),
            "url" => add_cors(encode_station_url(connection, connection.get_station_by_id_or_uuid(parameter), &ip, format)),
            "stations" => {
                match parameter {
                    "topvote" => add_cors(encode_stations(connection.get_stations_topvote(999999), format)),
                    "topclick" => add_cors(encode_stations(connection.get_stations_topclick(999999), format)),
                    "lastclick" => add_cors(encode_stations(connection.get_stations_lastclick(999999), format)),
                    "lastchange" => add_cors(encode_stations(connection.get_stations_lastchange(999999), format)),
                    "broken" => add_cors(encode_stations(connection.get_stations_broken(999999), format)),
                    "improvable" => add_cors(encode_stations(connection.get_stations_improvable(999999), format)),
                    "deleted" => add_cors(encode_stations(connection.get_stations_deleted_all(param_limit), format)),
                    "changed" => add_cors(encode_changes(get_changes(&request, &connection, None), format)),
                    "byurl" => add_cors(encode_stations(connection.get_stations_by_column_multiple("Url", param_url,true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                    "search" => add_cors(encode_stations(connection.get_stations_advanced(param_name, param_name_exact, param_country, param_country_exact, param_state, param_state_exact, param_language, param_language_exact, param_tag, param_tag_exact, param_tag_list, param_bitrate_min, param_bitrate_max, &param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                    _ => rouille::Response::empty_404()
                }
            },
            "checks" => add_cors(encode_checks(connection.get_checks(Some(parameter.to_string()), param_seconds), format)),
            _ => rouille::Response::empty_404()
        }
    } else if items.len() == 5 {
        let format = items[1];
        let command = items[2];
        let parameter = items[3];
        let search = items[4];
        if format == "v2" {
            // deprecated
            let format = command;
            let command = parameter;
            match command {
                "url" => add_cors(encode_station_url(connection, connection.get_station_by_id_or_uuid(search), &ip, format)),
                _ => rouille::Response::empty_404(),
            }
        }else{
            match command {
                "states" => add_cors(encode_states(get_states_with_parse(&request, &connection, Some(String::from(parameter)), Some(String::from(search))), format)),
                
                "stations" => {
                    match parameter {
                        "topvote" => add_cors(encode_stations(connection.get_stations_topvote(search.parse().unwrap_or(0)), format)),
                        "topclick" => add_cors(encode_stations(connection.get_stations_topclick(search.parse().unwrap_or(0)), format)),
                        "lastclick" => add_cors(encode_stations(connection.get_stations_lastclick(search.parse().unwrap_or(0)), format)),
                        "lastchange" => add_cors(encode_stations(connection.get_stations_lastchange(search.parse().unwrap_or(0)), format)),
                        "broken" => add_cors(encode_stations(connection.get_stations_broken(search.parse().unwrap_or(0)), format)),
                        "improvable" => add_cors(encode_stations(connection.get_stations_improvable(search.parse().unwrap_or(0)), format)),
                        "deleted" => add_cors(encode_stations(connection.get_stations_deleted(param_limit, search), format)),
                        "byname" => add_cors(encode_stations(connection.get_stations_by_column("Name", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bynameexact" => add_cors(encode_stations(connection.get_stations_by_column("Name", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bycodec" => add_cors(encode_stations(connection.get_stations_by_column("Codec", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bycodecexact" => add_cors(encode_stations(connection.get_stations_by_column("Codec", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bycountry" => add_cors(encode_stations(connection.get_stations_by_column("Country", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bycountryexact" => add_cors(encode_stations(connection.get_stations_by_column("Country", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bystate" => add_cors(encode_stations(connection.get_stations_by_column("Subcountry", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bystateexact" => add_cors(encode_stations(connection.get_stations_by_column("Subcountry", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bytag" => add_cors(encode_stations(connection.get_stations_by_column_multiple("Tags", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bytagexact" => add_cors(encode_stations(connection.get_stations_by_column_multiple("Tags", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bylanguage" => add_cors(encode_stations(connection.get_stations_by_column_multiple("Language", search.to_string(),false,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "bylanguageexact" => add_cors(encode_stations(connection.get_stations_by_column_multiple("Language", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "byuuid" => add_cors(encode_stations(connection.get_stations_by_column("StationUuid", search.to_string(),true,&param_order,param_reverse,param_hidebroken,param_offset,param_limit), format)),
                        "byid" => {
                            let id = search.parse();
                            match id{
                                Ok(i) => add_cors(encode_stations(connection.get_stations_by_id(i), format)),
                                Err(_) => add_cors(rouille::Response::empty_400())
                            }
                        },
                        "changed" => add_cors(encode_changes(get_changes(&request, &connection, Some(search.to_string())), format)),
                        _ => rouille::Response::empty_404()
                    }
                },
                _ => rouille::Response::empty_404()
            }
        }
    } else {
        rouille::Response::empty_404()
    }
}