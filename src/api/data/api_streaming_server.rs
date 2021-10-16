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
    fn serialize_servers<I, P>(servers: I) -> std::io::Result<String>
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
            if let Some(statusurl) = server.statusurl {
                xml.elem_text("statusurl", &statusurl)?;
            }
            if let Some(error) = server.error {
                xml.elem_text("error", &error)?;
            }
            if let Some(status) = server.status {
                xml.begin_elem("status")?;
                xml.cdata(&status)?;
                xml.end_elem()?;
            }
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response<I, P>(servers: I, format: &str) -> Result<ApiResponse, Box<dyn Error>>
    where
        I: IntoIterator<Item = P> + Serialize,
        P: Into<ApiStreamingServer>,
    {
        let servers: Vec<ApiStreamingServer> =
            servers.into_iter().map(|server| server.into()).collect();
        Ok(match format {
            "json" => ApiResponse::Text(serde_json::to_string(&servers)?),
            "xml" => ApiResponse::Text(ApiStreamingServer::serialize_servers(servers)?),
            _ => ApiResponse::UnknownContentType,
        })
    }
}
