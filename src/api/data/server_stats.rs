use crate::api::ApiResponse;
use crate::db::models::DbStreamingServer;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct ApiStreamingServer {
    pub uuid: String,
    pub url: String,
    pub statusurl: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

impl From<DbStreamingServer> for ApiStreamingServer {
    fn from(item: DbStreamingServer) -> Self {
        ApiStreamingServer {
            uuid: item.uuid,
            url: item.url,
            statusurl: item.statusurl,
            status: item.status,
            error: item.error,
        }
    }
}

impl ApiStreamingServer {
    pub fn serialize_servers<I,P>(servers: I) -> std::io::Result<String>
    where
        I: IntoIterator<Item = P>,
        P: Into<ApiStreamingServer>,
    {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("streamingservers")?;
        for server in servers {
            let server: ApiStreamingServer = server.into();
            xml.begin_elem("streamingserver")?;
            xml.elem_text("uuid", &server.uuid.to_string())?;
            xml.elem_text("url", &server.url.to_string())?;
            //xml.elem_text("statusurl", &server.statusurl.to_string())?;
            //xml.elem_text("error", &server.error.to_string())?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response<I,P>(server: I, format: &str) -> Result<ApiResponse, Box<dyn Error>>
    where
        I: IntoIterator<Item = P> + Serialize,
        P: Into<ApiStreamingServer>,
    {
        Ok(match format {
            "json" => ApiResponse::Text(serde_json::to_string(&server)?),
            "xml" => ApiResponse::Text(ApiStreamingServer::serialize_servers(server)?),
            _ => ApiResponse::UnknownContentType,
        })
    }
}
