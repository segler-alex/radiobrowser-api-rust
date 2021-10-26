use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone, Deserialize)]
struct DataMappingItem {
    from: String,
    to: String,
}

pub fn read_map_csv_file(file_path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    debug!("read_map_csv_file()");
    match Url::parse(file_path) {
        Ok(url) => {
            debug!("Remote url: {}", url);
            read_map_csv_file_reader(reqwest::blocking::get(url)?)
        }
        Err(_) => {
            debug!("Local path: {}", file_path);
            read_map_csv_file_reader(File::open(file_path)?)
        }
    }
}

fn read_map_csv_file_reader<R>(reader: R) -> Result<HashMap<String, String>, Box<dyn Error>>
where
    R: Read,
{
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .comment(Some(b'#'))
        .from_reader(reader);
    let mut r: HashMap<String, String> = HashMap::new();
    for result in rdr.deserialize() {
        let record: DataMappingItem = result?;
        trace!("loaded record: {:?}", record);
        if !r.contains_key(&record.from) {
            r.insert(record.from, record.to);
        } else {
            error!("Duplicate key in file: {}", record.from);
        }
    }
    Ok(r)
}
