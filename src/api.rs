extern crate rouille;

extern crate serde;
extern crate serde_json;
extern crate dns_lookup;

use std;
use db;
use self::dns_lookup::lookup_host;
use self::dns_lookup::lookup_addr;

#[derive(Serialize, Deserialize)]
pub struct ServerEntry {
    ip: String,
    name: String
}

#[derive(Serialize, Deserialize)]
struct Status {
    status: String
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

fn get_changes(request: &rouille::Request, connection: &db::Connection, station_id : Option<String>) -> Vec<db::Station>{
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

fn get_tags_with_parse(request: &rouille::Request, connection: &db::Connection, filter_prev : Option<String>) -> Vec<db::Tag>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let tags = connection.get_tags(filter, order, reverse, hidebroken);
    tags
}

/*fn encode_result1n_xml_single(entry: db::Result1n) -> String{
    encode_result1n_xml
}*/

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
        _ => rouille::Response::empty_406()
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

fn encode_tags(list : Vec<db::Tag>, format : &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        "xml" => {
            let j = db::serialize_tag_list(list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
        },
        _ => rouille::Response::empty_406()
    }
}

fn encode_status(status: Status, format : &str) -> rouille::Response {
    println!("status");
    match format {
        "json" => {
            let j = serde_json::to_string(&status).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
        },
        _ => rouille::Response::empty_406()
    }
}

pub fn run(connection: db::Connection, host : String, port : i32, threads : usize) {
    let listen_str = format!("{}:{}", host, port);
    println!("Listen on {} with {} threads", listen_str, threads);
    let x : Option<usize> = Some(threads);
    rouille::start_server_with_pool(listen_str, x, move |request| {
    //rouille::start_server(listen_str, move |request| {
        //rouille::log(&request, std::io::stdout(), || {
            handle_connection(&connection, request)
        //})
    });
}

fn get_status() -> Status {
    Status{status: "OK".to_string()}
}

fn handle_connection(connection: &db::Connection, request: &rouille::Request) -> rouille::Response {
    if request.method() != "POST" && request.method() != "GET" {
        return rouille::Response::empty_404();
    }

    let parts : Vec<&str> = request.raw_url().split('?').collect();
    let items : Vec<&str> = parts[0].split('/').collect();
    if items.len() == 2 {
        add_cors(encode_status(get_status(), "json"))
    } else if items.len() == 3 {
        let format = items[1];
        let command = items[2];
        let filter : Option<String> = None;

        match command {
            "languages" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Language", filter), format)),
            "countries" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Country", filter), format)),
            "states" => add_cors(encode_states(get_states_with_parse(&request, &connection, None, filter), format)),
            "codecs" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Codec", filter), format)),
            "tags" => add_cors(encode_tags(get_tags_with_parse(&request, &connection, filter), format)),
            "stations" => add_cors(encode_stations(connection.get_stations_by_all(), format)),
            "servers" => add_cors(dns_resolve(format)),
            "" => add_cors(encode_status(get_status(), "json")),
            _ => rouille::Response::empty_404()
        }
    } else if items.len() == 4 {
        let format = items[1];
        let command = items[2];
        let parameter = items[3];

        match command {
            "languages" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Language", Some(String::from(parameter))), format)),
            "countries" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Country", Some(String::from(parameter))), format)),
            "codecs" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Codec", Some(String::from(parameter))), format)),
            "tags" => add_cors(encode_tags(get_tags_with_parse(&request, &connection, Some(String::from(parameter))), format)),
            "states" => add_cors(encode_states(get_states_with_parse(&request, &connection, None, Some(String::from(parameter))), format)),
            "stations" => {
                match parameter {
                    "topvote" => add_cors(encode_stations(connection.get_stations_topvote(), format)),
                    "topclick" => add_cors(encode_stations(connection.get_stations_topclick(), format)),
                    "lastclick" => add_cors(encode_stations(connection.get_stations_lastclick(), format)),
                    "lastchange" => add_cors(encode_stations(connection.get_stations_lastchange(), format)),
                    "changed" => add_cors(encode_stations(get_changes(&request, &connection, None), format)),
                    _ => rouille::Response::empty_404()
                }
            },
            _ => rouille::Response::empty_404()
        }
    } else if items.len() == 5 {
        let format = items[1];
        let command = items[2];
        let parameter = items[3];
        let search = items[4];
        match command {
            "states" => add_cors(encode_states(get_states_with_parse(&request, &connection, Some(String::from(parameter)), Some(String::from(search))), format)),
            "stations" => {
                match parameter {
                    "byname" => add_cors(encode_stations(connection.get_stations_by_name(search.to_string()), format)),
                    "byid" => {
                        let id = search.parse();
                        match id{
                            Ok(i) => add_cors(encode_stations(connection.get_stations_by_id(i), format)),
                            Err(_) => add_cors(rouille::Response::empty_400())
                        }
                    },
                    "changed" => add_cors(encode_stations(get_changes(&request, &connection, Some(search.to_string())), format)),
                    _ => rouille::Response::empty_404()
                }
            },
            _ => rouille::Response::empty_404()
        }
    } else {
        rouille::Response::empty_404()
    }
}