extern crate rouille;
extern crate mysql;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate dns_lookup;

use std::io;
use std::env;
use std::{thread, time};

mod db;
mod api;

fn get_1_n_with_parse(request: &rouille::Request, pool: &mysql::Pool, column: &str, filter_prev : Option<String>) -> Vec<db::Result1n>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let stations = db::get_1_n(&pool, column, filter, order, reverse, hidebroken);
    stations
}

fn encode_other(list : Vec<db::Result1n>, format : &str) -> rouille::Response {
    match format {
        "json" => {
            let j = serde_json::to_string(&list).unwrap();
            rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
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
        _ => rouille::Response::empty_404()
    }
}

fn myrun(pool : mysql::Pool) {
    rouille::start_server("0.0.0.0:8080", move |request| {
        rouille::log(&request, io::stdout(), || {
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
                    "languages" => api::add_cors(encode_other(get_1_n_with_parse(&request, &pool, "Language", filter), format)),
                    "countries" => api::add_cors(encode_other(get_1_n_with_parse(&request, &pool, "Country", filter), format)),
                    "codecs" => api::add_cors(encode_other(get_1_n_with_parse(&request, &pool, "Codec", filter), format)),
                    "stations" => api::add_cors(encode_stations(db::get_stations(&pool, filter), format)),
                    "servers" => api::add_cors(api::dns_resolve(format)),
                    _ => rouille::Response::empty_404()
                };
                result
            } else {
                rouille::Response::empty_404()
            }
        })
    });
}

fn main() {
    println!("Listening on 8080");
    let dbhost = env::var("DB_HOST").unwrap_or(String::from("localhost"));
    let dbport = env::var("DB_PORT").unwrap_or(String::from("3306"));
    let dbuser = env::var("DB_USER").expect("You have to set DB_USER env var");
    let dbpass = env::var("DB_PASS").expect("You have to set DB_PASS env var");
    let dbname = env::var("DB_NAME").expect("You have to set DB_NAME env var");
    
    let mut counter : i32 = 0;
    loop {
        let connection_string = format!("mysql://{}:{}@{}:{}/{}",dbuser,dbpass,dbhost,dbport,dbname);
        println!("Connection string: {}", connection_string);
        let pool = mysql::Pool::new(connection_string);
        match pool {
            Ok(v) => {
                myrun(v);
                break;
            },
            Err(_) => {
                counter = counter + 1;
                if counter < 10 {
                    thread::sleep(time::Duration::from_millis(1000));
                }else{
                    break;
                }
            }
        }
    }
}
