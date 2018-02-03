extern crate rouille;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::{io, env, thread, time};

mod db;
mod api;

fn get_1_n_with_parse(request: &rouille::Request, connection: &db::Connection, column: &str, filter_prev : Option<String>) -> Vec<db::Result1n>{
    let filter = request.get_param("filter").or(filter_prev);
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let stations = connection.get_1_n(column, filter, order, reverse, hidebroken);
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

fn myrun(pool: db::Connection, port : i32) {
    let listen_str = format!("0.0.0.0:{}", port);
    println!("Listen on {}", listen_str);
    rouille::start_server(listen_str, move |request| {
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
                    "stations" => api::add_cors(encode_stations(pool.get_stations(filter), format)),
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
    let listen_port : i32 = env::var("PORT").unwrap_or(String::from("8080")).parse().expect("listen port is not number");
    let dbhost = env::var("DB_HOST").unwrap_or(String::from("localhost"));
    let dbport : i32 = env::var("DB_PORT").unwrap_or(String::from("3306")).parse().expect("db port is not a number");
    let dbuser = env::var("DB_USER").expect("You have to set DB_USER env var");
    let dbpass = env::var("DB_PASS").expect("You have to set DB_PASS env var");
    let dbname = env::var("DB_NAME").expect("You have to set DB_NAME env var");
    
    let mut counter : i32 = 0;
    loop {
        let connection = db::new(&dbhost, dbport, &dbname, &dbuser, &dbpass);
        match connection {
            Ok(v) => {
                myrun(v, listen_port);
                break;
            },
            Err(e) => {
                println!("{}", e);
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
