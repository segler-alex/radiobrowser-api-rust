extern crate mysql;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Station {
    station_id: i32,
    name: String,
    url: String,
    homepage: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Result1n {
    name: String,
    value: String,
    stationcount: u32,
}

pub fn get_stations(pool: &mysql::Pool, search: Option<String>) -> Vec<Station>{
    let query : String;
    match search{
        Some(value) => {
            query = format!("SELECT StationID,Name,Url,Homepage from Station WHERE Name LIKE '%{search}%' ORDER BY Name", search = value);
        },
        None => {
            query = format!("SELECT StationID,Name,Url,Homepage from Station ORDER BY Name");
        }
    }
    println!("{}",query);

    let stations: Vec<Station> =
    pool.prep_exec(query, ())
    .map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let (station_id, name, url, homepage) = mysql::from_row(row);
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

pub fn get_1_n(pool: &mysql::Pool, column: &str, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<Result1n>{
    let query : String;
    let reverse_string = if reverse { "DESC" } else { "ASC" };
    let hidebroken_string = if hidebroken { " AND LastCheckOK=TRUE" } else { "" };
    match search{
        Some(value) => {
            query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM Station WHERE {column} LIKE '%{search}%' AND {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, search = value, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
        },
        None => {
            query = format!("SELECT {column} AS value,{column},COUNT(*) AS stationcount FROM Station WHERE {column}<>'' {hidebroken} GROUP BY {column} ORDER BY {order} {reverse}", column = column, order = order, reverse = reverse_string, hidebroken = hidebroken_string);
        }
    }

    let stations: Vec<Result1n> =
    pool.prep_exec(query, ())
    .map(|result| {
        result.map(|x| x.unwrap()).map(|row| {
            let (name, value, stationcount) = mysql::from_row(row);
            Result1n {
                name: name,
                value: value,
                stationcount: stationcount,
            }
        }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
    }).unwrap(); // Unwrap `Vec<Payment>`

    stations
}