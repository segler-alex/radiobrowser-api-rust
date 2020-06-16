use std::error::Error;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct AllParameters {
    #[serde(rename = "u")]
    pub url: String,
    #[serde(rename = "ul")]
    pub param_uuids: Vec<String>,
    #[serde(rename = "ts")]
    pub param_tags: Option<String>,
    #[serde(rename = "hp")]
    pub param_homepage: Option<String>,
    #[serde(rename = "fv")]
    pub param_favicon: Option<String>,
    #[serde(rename = "aid")]
    pub param_last_changeuuid: Option<String>,
    #[serde(rename = "eid")]
    pub param_last_checkuuid: Option<String>,
    #[serde(rename = "iid")]
    pub param_last_clickuuid: Option<String>,
    #[serde(rename = "na")]
    pub param_name: Option<String>,
    #[serde(rename = "nx")]
    pub param_name_exact: bool,
    #[serde(rename = "c")]
    pub param_country: Option<String>,
    #[serde(rename = "cx")]
    pub param_country_exact: bool,
    #[serde(rename = "cc")]
    pub param_countrycode: Option<String>,
    #[serde(rename = "st")]
    pub param_state: Option<String>,
    #[serde(rename = "sx")]
    pub param_state_exact: bool,
    #[serde(rename = "lg")]
    pub param_language: Option<String>,
    #[serde(rename = "lx")]
    pub param_language_exact: bool,
    #[serde(rename = "tg")]
    pub param_tag: Option<String>,
    #[serde(rename = "tx")]
    pub param_tag_exact: bool,
    #[serde(rename = "tl")]
    pub param_tag_list: Vec<String>,
    #[serde(rename = "co")]
    pub param_codec: Option<String>,
    #[serde(rename = "bi")]
    pub param_bitrate_min: u32,
    #[serde(rename = "ba")]
    pub param_bitrate_max: u32,
    #[serde(rename = "or")]
    pub param_order: String,
    #[serde(rename = "re")]
    pub param_reverse: bool,
    #[serde(rename = "hb")]
    pub param_hidebroken: bool,
    #[serde(rename = "of")]
    pub param_offset: u32,
    #[serde(rename = "li")]
    pub param_limit: u32,
    #[serde(rename = "se")]
    pub param_seconds: u32,
    #[serde(rename = "up")]
    pub param_url: Option<String>,
}

impl AllParameters {
    pub fn to_string(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(&self)?)
    }
}
