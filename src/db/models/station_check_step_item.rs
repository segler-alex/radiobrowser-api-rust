use chrono::DateTime;
use chrono::Utc;
use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct StationCheckStepItem {
    pub id: u32,
    pub stepuuid: String,
    pub parent_stepuuid: String,
    pub checkuuid: String,
    pub stationuuid: String,
    pub urltype: String,
    pub error: String,
    pub inserttime: DateTime<Utc>,
}