use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct DbStreamingServerNew {
    pub url: String,
    pub statusurl: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

impl DbStreamingServerNew {
    pub fn new(url: String, statusurl: Option<String>, status: Option<String>, error: Option<String>) -> Self {
        DbStreamingServerNew {
            url,
            statusurl,
            status,
            error,
        }
    }
}
