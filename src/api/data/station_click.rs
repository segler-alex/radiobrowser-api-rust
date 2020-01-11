use crate::db::models::StationClickItem;
use std::convert::TryFrom;
use std::error::Error;

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationClickV0 {
    pub stationuuid: String,
    pub clickuuid: String,
    pub clicktimestamp: String,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct StationClick {
    pub stationuuid: String,
    pub clickuuid: String,
    pub clicktimestamp: String,
}

impl StationClick {
    pub fn new(
        stationuuid: String,
        clickuuid: String,
        clicktimestamp: String,
    ) -> Self {
        StationClick {
            stationuuid,
            clickuuid,
            clicktimestamp,
        }
    }

    pub fn serialize_station_checks(entries: Vec<StationClick>) -> std::io::Result<String> {
        let mut xml = xml_writer::XmlWriter::new(Vec::new());
        xml.begin_elem("result")?;
        for entry in entries {
            xml.begin_elem("click")?;
            xml.attr_esc("stationuuid", &entry.stationuuid)?;
            xml.attr_esc("clickuuid", &entry.clickuuid)?;
            xml.attr_esc("clicktimestamp", &entry.clicktimestamp)?;
            xml.end_elem()?;
        }
        xml.end_elem()?;
        xml.close()?;
        xml.flush()?;
        Ok(String::from_utf8(xml.into_inner()).unwrap_or("encoding error".to_string()))
    }

    pub fn get_response(list: Vec<StationClick>, format: &str) -> Result<rouille::Response, Box<dyn Error>> {
        Ok(match format {
            "json" => {
                let j = serde_json::to_string(&list)?;
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","application/json")
            },
            "xml" => {
                let j = StationClick::serialize_station_checks(list)?;
                rouille::Response::text(j).with_no_cache().with_unique_header("Content-Type","text/xml")
            },
            _ => rouille::Response::empty_406()
        })
    }
}

impl TryFrom<StationClickV0> for StationClick {
    type Error = Box<dyn std::error::Error>;

    fn try_from(item: StationClickV0) -> Result<Self, Self::Error> {
        Ok(StationClick {
            stationuuid: item.stationuuid,
            clickuuid: item.clickuuid,
            clicktimestamp: item.clicktimestamp,
        })
    }
}

impl From<StationClickItem> for StationClick {
    fn from(item: StationClickItem) -> Self {
        StationClick::new(
            item.stationuuid,
            item.clickuuid,
            item.clicktimestamp,
        )
    }
}