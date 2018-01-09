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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Station {
    station_id: i32,
    name: String,
    url: String,
    homepage: String,
}

fn get_stations(pool: &mysql::Pool) -> Vec<Station>{
    let stations: Vec<Station> =
    pool.prep_exec("SELECT StationID,Name,Url,Homepage from radio.Station ORDER BY Name LIMIT 10", ())
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

fn main() {
    println!("Listening on 8080");
    let pool = my::Pool::new("mysql://radiouser:password@dbserver:3306").unwrap();

    rouille::start_server("0.0.0.0:8080", move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    rouille::Response::text("hello world!")
                },

                (GET) (/stations) => {
                    let stations = get_stations(&pool);
                    let j = serde_json::to_string(&stations).unwrap();
                    rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
                },

                _ => rouille::Response::empty_404()
            )
        })
    });
}
