extern crate mysql;
extern crate xml_writer;

use std;

pub struct Connection {
    pool: mysql::Pool
}

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

pub fn serialize_result1n_list(type_str: &str, entries: Vec<Result1n>) -> std::io::Result<String> {
    let mut xml = xml_writer::XmlWriter::new(Vec::new());
    xml.begin_elem("result")?;
    for entry in entries{
        xml.begin_elem(type_str)?;
            xml.attr_esc("name", &entry.name)?;
            xml.attr_esc("value", &entry.value)?;
            let count_str = format!("{}", entry.stationcount);
            xml.attr_esc("stationcount", &count_str)?;
        xml.end_elem()?;
    }
    xml.end_elem()?;
    xml.close()?;
    xml.flush()?;
    Ok(String::from_utf8(xml.into_inner()).unwrap())
}

impl Connection {
    pub fn get_stations(&self, search: Option<String>) -> Vec<Station>{
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
        self.pool.prep_exec(query, ())
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

    pub fn get_1_n(&self, column: &str, search: Option<String>, order : String, reverse : bool, hidebroken : bool) -> Vec<Result1n>{
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
        self.pool.prep_exec(query, ())
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
}
pub enum DBError{
    ConnectionError (String)
}

impl std::fmt::Display for DBError{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        match *self{
            DBError::ConnectionError(ref v) => write!(f, "{}", v)
        }
    }
}

pub fn new(host: &String,port : i32, name: &String, user: &String, password: &String) -> Result<Connection, DBError> {
    let connection_string = format!("mysql://{}:{}@{}:{}/{}",user,password,host,port,name);
    println!("Connection string: {}", connection_string);
    let pool = mysql::Pool::new(connection_string);
    match pool {
        Ok(p) => Ok(Connection{pool: p}),
        Err(e) => Err(DBError::ConnectionError(e.to_string()))
    }
}