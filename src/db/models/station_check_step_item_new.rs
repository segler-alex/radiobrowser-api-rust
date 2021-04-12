use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct StationCheckStepItemNew {
    pub stepuuid: String,
    pub parent_stepuuid: Option<String>,
    pub checkuuid: String,
    pub stationuuid: String,
    pub urltype: Option<String>,
    pub error: Option<String>,
}