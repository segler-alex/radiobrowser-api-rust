use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct DbStreamingServer {
    pub id: u32,
    pub uuid: String,
    pub url: String,
    pub statusurl: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

impl DbStreamingServer {
    pub fn new(id: u32, uuid: String, url: String, statusurl: Option<String>, status: Option<String>, error: Option<String>) -> Self {
        DbStreamingServer {
            id,
            uuid,
            url,
            statusurl,
            status,
            error,
        }
    }
}
