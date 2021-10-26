use std::str::ParseBoolError;
use crate::api::rouille::Request;
use std::collections::HashMap;
use std::io::Read;
use url::form_urlencoded;
//use self::serde_json::value::{Map};

pub struct RequestParameters{
    values: HashMap<String, String>
}

impl RequestParameters {
    pub fn new(req: &Request) -> Self {
        let mut values = HashMap::new();
        RequestParameters::decode(req, &mut values);
        RequestParameters{
            values
        }
    }

    fn decode(req: &Request, map: &mut HashMap<String,String>) {
        let content_type_raw: &str = req.header("Content-Type").unwrap_or("nothing");
        let content_type_arr: Vec<&str> = content_type_raw.split(";").collect();
        if content_type_arr.len() == 0{
            return;
        }
        let content_type = content_type_arr[0].trim();

        if req.method() == "POST" {
            match content_type {
                "multipart/form-data" =>{
                    RequestParameters::decode_form_data(req, map);
                },
                "application/x-www-form-urlencoded" => {
                    RequestParameters::decode_url_encoded(req, map);
                },
                "application/json" => {
                    RequestParameters::decode_json(req, map);
                },
                "nothing" => {
                    // ignore body
                },
                _ =>{
                    error!("unknown content type: {}", content_type);
                }
            }
        }

        RequestParameters::decode_url_query(req, map);
    }

    fn decode_url_query(req: &Request, map: &mut HashMap<String, String>) {
        let iter = form_urlencoded::parse(req.raw_query_string().as_bytes());
        for (key,val) in iter {
            trace!("application/x-www-form-urlencoded '{}' => '{}'", key, val);
            let key = String::from(key);
            if !map.contains_key(&key){
                map.insert(key, String::from(val));
            }
        }
    }

    fn decode_url_encoded(req: &Request, map: &mut HashMap<String, String>) {
        let mut data = req.data().unwrap();
        let mut buf = Vec::new();
        match data.read_to_end(&mut buf) {
            Ok(_) => {
                let iter = form_urlencoded::parse(&buf);
                for (key,val) in iter {
                    trace!("application/x-www-form-urlencoded '{}' => '{}'", key, val);
                    let key = String::from(key);
                    if !map.contains_key(&key){
                        map.insert(key, String::from(val));
                    }
                }
            }
            Err(_)=>{
                error!("err");
            }
        }
    }

    fn decode_form_data(req: &Request, map: &mut HashMap<String, String>) {
        let multipart = rouille::input::multipart::get_multipart_input(&req);
        match multipart {
            Ok(mut content)=>{
                loop{
                    let field = content.next();
                    if let Some(mut field) = field {
                        if field.is_text(){
                            let mut buf = String::new();
                            let res = field.data.read_to_string(&mut buf);
                            if let Ok(_) = res {
                                trace!("multipart/form-data '{}' => '{}'", field.headers.name, buf);
                                let key = String::from(&(*field.headers.name));
                                let val = buf;
                                if !map.contains_key(&key){
                                    map.insert(key, String::from(val));
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
            Err(_) => {
                error!("err");
            }
        };
    }

    fn decode_json(req: &Request, map: &mut HashMap<String, String>) {
        let data = req.data();
        if let Some(mut data) = data {
            let mut buf = Vec::new();
            match data.read_to_end(&mut buf) {
                Ok(_) => {
                    let v: Result<HashMap<String, serde_json::Value>, serde_json::error::Error> = serde_json::from_slice(&buf);
                    match v {
                        Err(_) =>{
                            error!("unable to decode json");
                        },
                        Ok(v) => {
                            for (key, value) in v {
                                trace!("application/json {} => {}", key, value);
                                if !map.contains_key(&key){
                                    if let Some(value) = value.as_u64() {
                                        map.insert(key, String::from(value.to_string()));
                                    }
                                    else if let Some(value) = value.as_i64() {
                                        map.insert(key, String::from(value.to_string()));
                                    }
                                    else if let Some(value) = value.as_f64() {
                                        map.insert(key, String::from(value.to_string()));
                                    }
                                    else if let Some(value) = value.as_str() {
                                        map.insert(key, String::from(value));
                                    }
                                    else if let Some(value) = value.as_bool() {
                                        map.insert(key, String::from(value.to_string()));
                                    }
                                    else if let Some(value) = value.as_array() {
                                        let list: Vec<String> = value.into_iter().map(|item| {
                                            if item.is_string(){
                                                item.as_str().unwrap().trim().to_string()
                                            }else{
                                                String::from("")
                                            }
                                        }).filter(|item| {
                                            item != ""
                                        }).collect();
                                        let value = list.join(",");
                                        map.insert(key, value);
                                    }
                                    else{
                                        error!("unsupported value type in json");
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    error!("error");
                }
            }
        }
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        let v = self.values.get(name);
        if let Some(v) = v {
            return Some(String::from(v));
        }
        None
    }

    pub fn get_bool(&self, name: &str, default: bool) -> bool {
        let v = self.values.get(name);
        if let Some(v) = v {
            let parsed = v.parse::<bool>();
            if let Ok(parsed) = parsed {
                return parsed;
            }
        }
        default
    }

    pub fn get_bool_opt(&self, name: &str) -> Result<Option<bool>, ParseBoolError> {
        let v = self.values.get(name);
        v.map(|v| v.parse::<bool>()).transpose()
    }

    pub fn get_number(&self, name: &str, default: u32) -> u32 {
        let v = self.values.get(name);
        if let Some(v) = v {
            let parsed = v.parse::<u32>();
            if let Ok(parsed) = parsed {
                return parsed;
            }else{
                error!("could not parse '{}'", v);
            }
        }
        default
    }

    pub fn get_double(&self, name: &str, default: Option<f64>) -> Option<f64> {
        let v = self.values.get(name);
        if let Some(v) = v {
            let parsed = v.parse::<f64>();
            if let Ok(parsed) = parsed {
                return Some(parsed);
            }else{
                error!("could not parse '{}'", v);
            }
        }
        default
    }
}