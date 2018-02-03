
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

fn get_1_n_with_parse(request: &rouille::Request, connection: &db::Connection, column: &str, filter_prev : Option<String>) -> Vec<db::Result1n>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let stations = connection.get_1_n(column, filter, order, reverse, hidebroken);
    stations
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
        _ => rouille::Response::empty_404()
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
        _ => rouille::Response::empty_404()
    }
}

pub fn run(connection: db::Connection, host : String, port : i32) {
    let listen_str = format!("{}:{}", host, port);
    println!("Listen on {}", listen_str);
    rouille::start_server(listen_str, move |request| {
        rouille::log(&request, std::io::stdout(), || {
            handle_connection(&connection, request)
        })
    });
}

fn handle_connection(connection: &db::Connection, request: &rouille::Request) -> rouille::Response {
    if request.method() != "POST" && request.method() != "GET" {
        return rouille::Response::empty_404();
    }
    let items : Vec<&str> = request.raw_url().split('/').collect();
    // println!("method: {} - {} - {} len={}",request.method(), request.raw_url(), items[1], items.len());

    if items.len() >= 3 && items.len() <= 4 {
        let format = items[1];
        let command = items[2];

        let filter : Option<String> = if items.len() >= 4 {Some(String::from(items[3]))} else {None};
        let result = match command {
            "languages" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Language", filter), format)),
            "countries" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Country", filter), format)),
            "codecs" => add_cors(encode_result1n(command, get_1_n_with_parse(&request, &connection, "Codec", filter), format)),
            "stations" => add_cors(encode_stations(connection.get_stations(filter), format)),
            "servers" => add_cors(dns_resolve(format)),
            _ => rouille::Response::empty_404()
        };
        result
    } else {
        rouille::Response::empty_404()
    }
}