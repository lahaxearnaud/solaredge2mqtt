use serde::{Serialize, Deserialize};


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(rename = "device_class")]
    pub device_class: String,
    #[serde(rename = "state_topic")]
    pub state_topic: String,
    #[serde(rename = "unit_of_measurement")]
    pub unit_of_measurement: String,
    #[serde(rename = "value_template")]
    pub value_template: String,
    #[serde(rename = "unique_id")]
    pub unique_id: String,
    pub device: Device,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub identifiers: Vec<String>,
    pub name: String,
    pub manufacturer: String,
}


pub struct Sensor {
    pub technical_name: String,
    pub name: String,
}