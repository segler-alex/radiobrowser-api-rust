use chrono::DateTime;
use chrono::Utc;
use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct StationCheckStepItem {
    pub id: u32,
    pub stepuuid: String,
    pub parent_stepuuid: Option<String>,
    pub checkuuid: String,
    pub stationuuid: String,
    pub url: String,
    pub urltype: Option<String>,
    pub error: Option<String>,
    pub inserttime: DateTime<Utc>,
}