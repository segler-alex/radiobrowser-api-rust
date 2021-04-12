use serde::{Serialize,Deserialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct StationCheckStepItemNew {
    pub stepuuid: String,
    pub parent_stepuuid: String,
    pub checkuuid: String,
    pub stationuuid: String,
    pub urltype: String,
    pub error: String,
}