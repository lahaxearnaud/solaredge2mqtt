use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub site_current_power_flow: SiteCurrentPowerFlow,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteCurrentPowerFlow {
    pub update_refresh_rate: i64,
    pub unit: String,
    pub connections: Vec<Connection>,
    #[serde(rename = "GRID")]
    pub grid: Item,
    #[serde(rename = "LOAD")]
    pub load: Item,
    #[serde(rename = "PV")]
    pub pv: Item,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub from: String,
    pub to: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub status: String,
    pub current_power: f64,
}
