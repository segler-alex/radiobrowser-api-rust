extern crate rouille;

extern crate serde;
extern crate serde_json;
extern crate dns_lookup;

use std;
use self::dns_lookup::lookup_host;
use self::dns_lookup::lookup_addr;

#[derive(Serialize, Deserialize)]
pub struct ServerEntry {
    ip: String,
    name: String
}

pub fn add_cors(result : rouille::Response) -> rouille::Response {
    result.with_unique_header("Access-Control-Allow-Origin", "*")
        .with_unique_header("Access-Control-Allow-Headers", "origin, x-requested-with, content-type")
        .with_unique_header("Access-Control-Allow-Methods", "GET,POST")
}

pub fn dns_resolve(format : &str) -> rouille::Response {
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