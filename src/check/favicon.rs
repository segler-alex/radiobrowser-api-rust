/*
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use std::time::Duration;
use website_icon_extract;

pub fn check(
    homepage: &str,
    old_favicon: &str,
    useragent: &str,
    timeout: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let check = check_url(old_favicon, useragent, timeout);
    if !check {
        debug!("Check for favicon: {}", homepage);
        let icons = website_icon_extract::extract_icons(homepage, useragent, timeout)?;
        if icons.len() > 0 {
            debug!("Favicon {}", icons[0]);
            return Ok(icons[0].clone());
        } else {
            debug!("No favicons found for: {}", homepage);
        }
        Ok(String::from(""))
    } else {
        Ok(String::from(old_favicon))
    }
}

fn check_url(url: &str, useragent: &str, timeout: u32) -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout.into()))
        .build();
    if client.is_err() {
        return false;
    }
    let client = client.unwrap();
    let res = client
        .get(url)
        .header(USER_AGENT, useragent.to_string())
        .send();
    match res {
        Ok(r) => {
            if r.status().is_success() {
                let t = r.headers().get(CONTENT_TYPE);
                match t {
                    Some(t) => {
                        let value = t.to_str();
                        if let Ok(value) = value {
                            if value.starts_with("image"){
                                return true;
                            }
                        }
                    }
                    None => {
                        return false;
                    }
                }
            }
            return false;
        }
        Err(_) => {
            return false;
        }
    }
}
*/