#[macro_use]
extern crate rouille;
#[macro_use]
extern crate mysql;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::io;
use mysql as my;
use std::env;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Station {
    station_id: i32,
    name: String,
    url: String,
    homepage: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Result1n {
    name: String,
    value: String,
    stationcount: u32,
}

fn get_stations(pool: &mysql::Pool, search: Option<String>) -> Vec<Station>{
    let query : String;
    match search{
        Some(value) => {
            query = format!("SELECT StationID,Name,Url,Homepage from radio.Station WHERE Name LIKE '%{search}%' ORDER BY Name", search = value);
        },
        None => {
            query = format!("SELECT StationID,Name,Url,Homepage from radio.Station ORDER BY Name");
        }
    }
    println!("{}",query);

    let stations: Vec<Station> =
    pool.prep_exec(query, ())
    .map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let (station_id, name, url, homepage) = my::from_row(row);
            Station {
                station_id: station_id,
                name: name,
                url: url,
                homepage: homepage,
            }
        }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
    }).unwrap(); // Unwrap `Vec<Payment>`

    stations
}

fn get_1_n_with_parse(request: &rouille::Request, pool: &mysql::Pool, column: &str, filter_prev : Option<String>) -> Vec<Result1n>{
    let filter : String = request.get_param("filter").unwrap_or(filter_prev.unwrap_or(String::from("")));
    let order : String = request.get_param("order").unwrap_or(String::from("value"));
    let reverse : bool = request.get_param("reverse").unwrap_or(String::from("false")) == "true";
    let hidebroken : bool = request.get_param("hidebroken").unwrap_or(String::from("false")) == "true";
    let stations = get_1_n(&pool, column, Some(filter), order, reverse, hidebroken);
    stations
}

fn get_1_n(pool: &mysql::Pool, column: &str, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<Result1n>{
    let query : String;
    let reverse_string = if reverse { "DESC" } else { "ASC" };
    let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
    match search{
        Some(value) => {
            query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM radio.Station WHERE {column} LIKE '%{search}%' AND {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, search = value, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
        },
        None => {
            query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM radio.Station WHERE {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
        }
    }

    let stations: Vec<Result1n> =
    pool.prep_exec(query, ())
    .map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let (name, value, stationcount) = my::from_row(row);
            Result1n {
                name: name,
                value: value,
                stationcount: stationcount,
            }
        }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
    }).unwrap(); // Unwrap `Vec<Payment>`

    stations
}

fn main() {
    println!("Listening on 8080");
    let dbhost = env::var("DB_HOST").unwrap_or(String::from("localhost"));
    let dbport = env::var("DB_PORT").unwrap_or(String::from("3306"));
    let dbuser = env::var("DB_USER").expect("You have to set DB_USER env var");
    let dbpass = env::var("DB_PASS").expect("You have to set DB_PASS env var");
    let connection_string = format!("mysql://{}:{}@{}:{}",dbuser,dbpass,dbhost,dbport);
    println!("Connection string: {}", connection_string);
    let pool = my::Pool::new(connection_string).unwrap();

    rouille::start_server("0.0.0.0:8080", move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    rouille::Response::text("hello world!")
                },

                (GET) (/{format : String}/stations) => {
                    let stations = get_stations(&pool, None);
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                (GET) (/{format : String}/stations/{search : String}) => {
                    let stations = get_stations(&pool, Some(search));
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                (GET) (/{format : String}/languages) => {
                    let stations = get_1_n_with_parse(&request, &pool, "Language", None);
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                (GET) (/{format : String}/countries) => {
                    let stations = get_1_n_with_parse(&request, &pool, "Country", None);
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                (GET) (/{format : String}/codecs) => {
                    let stations = get_1_n_with_parse(&request, &pool, "Codec", None);
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                (GET) (/{format : String}/countries/{filter : String}) => {
                    let stations = get_1_n_with_parse(&request, &pool, "Country", Some(filter));
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                (GET) (/{format : String}/codecs/{filter : String}) => {
                    let stations = get_1_n_with_parse(&request, &pool, "Codec", Some(filter));
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                (GET) (/{format : String}/languages/{filter : String}) => {
                    let stations = get_1_n_with_parse(&request, &pool, "Language", Some(filter));
                    let j = serde_json::to_string(&stations).unwrap();
                    if format == "json" {
                        rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                    }else{
                        rouille::Response::empty_404()
                    }
                },

                _ => rouille::Response::empty_404()
            )
        })
    });
}
