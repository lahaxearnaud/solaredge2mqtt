mod solaredge;
mod homeassistant;
mod mqtt;

extern crate dotenv;

use mqtt::state::State;
use homeassistant::config::{Config, Device, Sensor};

use std::env;
use solaredge::current_power_flow::{Root, SiteCurrentPowerFlow};
use dotenv::dotenv;
use simple_logger::SimpleLogger;
use reqwest;
use tokio::time::Duration;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json::json;

const ENV_ERROR_MESSAGE: &'static str = "env var is missing";

const ENV_API_KEY: &'static str = "API_KEY";
const ENV_SITE_ID: &'static str = "SITE_ID";
const ENV_MQTT_HOST: &'static str = "MQTT_HOST";
const ENV_MQTT_PORT: &'static str = "MQTT_PORT";
const ENV_MQTT_CLIENT_NAME: &'static str = "MQTT_CLIENT_NAME";
const ENV_MQTT_USERNAME: &'static str = "MQTT_USERNAME";
const ENV_MQTT_PASSWORD: &'static str = "MQTT_PASSWORD";
const ENV_MQTT_HOMEASSISTANT_DISCOVERY_TOPIC: &'static str = "MQTT_HOMEASSISTANT_DISCOVERY_TOPIC";
const ENV_MQTT_HOMEASSISTANT_STATE_TOPIC: &'static str = "MQTT_HOMEASSISTANT_STATE_TOPIC";

const PROJECT_NAME: &'static str = "solaredge2mqtt";

const SOLAREDGE_UNIT: &'static str = "kW";
const SOLAREDGE_MONITORING_API_HOST: &'static str = "monitoringapi.solaredge.com";

const HOMEASSISTANT_DEVICE_CLASS: &'static str = "power";

#[tokio::main]
async fn main() {
    dotenv().ok();
    SimpleLogger::new().env().init().unwrap();

    let update_delay = env::var("DELAY_BETWEEN_UPDATE_S")
        .unwrap_or_else(|_| { panic!("API_KEY env var is missing") })
        .to_string()
        .parse::<u64>()
        .unwrap();

    let api_key = env::var(ENV_API_KEY)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_API_KEY, ENV_ERROR_MESSAGE) });
    let site_id = env::var(ENV_SITE_ID)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_SITE_ID, ENV_ERROR_MESSAGE) });
    let mqtt_client_name = env::var(ENV_MQTT_CLIENT_NAME)
        .unwrap_or("solaredge2mqtt".to_string());
    let mqtt_host = env::var(ENV_MQTT_HOST)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_MQTT_HOST, ENV_ERROR_MESSAGE) });
    let mqtt_port = env::var(ENV_MQTT_PORT)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_MQTT_PORT, ENV_ERROR_MESSAGE) })
        .parse::<u16>()
        .unwrap();
    let mqtt_username = env::var(ENV_MQTT_USERNAME)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_MQTT_USERNAME, ENV_ERROR_MESSAGE) });
    let mqtt_password = env::var(ENV_MQTT_PASSWORD)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_MQTT_PASSWORD, ENV_ERROR_MESSAGE) });
    let mqtt_homeassistant_discovery_topic = env::var(ENV_MQTT_HOMEASSISTANT_DISCOVERY_TOPIC)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_MQTT_HOMEASSISTANT_DISCOVERY_TOPIC, ENV_ERROR_MESSAGE) });
    let mqtt_homeassistant_state_topic = env::var(ENV_MQTT_HOMEASSISTANT_STATE_TOPIC)
        .unwrap_or_else(|_| { panic!("{} {}", ENV_MQTT_HOMEASSISTANT_STATE_TOPIC, ENV_ERROR_MESSAGE) });

    let mut mqttoptions = MqttOptions::new(mqtt_client_name, mqtt_host, mqtt_port);
    mqttoptions.set_credentials(mqtt_username, mqtt_password);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    generate_homeasitant_sensors(
        &site_id,
        mqtt_homeassistant_discovery_topic,
        &mqtt_homeassistant_state_topic,
        &mut client
    ).await;

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    log::debug!("Notification: {:?}", notification)
                }
                Err(e) => {
                    log::error!("Erreur: {:?}", e);
                }
            }
        }
    });

    let _ = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(update_delay));
        loop {
            interval.tick().await;
            let site_current_power_flow = update_from_solaredge(
                site_id.clone(),
                api_key.clone(),
            ).await;

            if site_current_power_flow.is_err() {
                log::error!("Failed to call solaredge api");
                continue;
            }

            let _ = publish_to_mqtt(
                &mut client,
                site_current_power_flow.unwrap(),
                site_id.clone(),
                mqtt_homeassistant_state_topic.clone(),
            ).await;
        }
    }).await;
}

async fn generate_homeasitant_sensors(
    site_id: &String,
    mqtt_homeassistant_discovery_topic: String,
    mqtt_homeassistant_state_topic: &String,
    mut client: &mut AsyncClient
) {
    let sensors = [
        Sensor {
            name: "PV to LOAD".to_string(),
            technical_name: "pv_to_load".to_string(),
        },
        Sensor {
            name: "LOAD to GRID".to_string(),
            technical_name: "load_to_grid".to_string(),
        },
        Sensor {
            name: "GRID to LOAD".to_string(),
            technical_name: "grid_to_load".to_string(),
        },
        Sensor {
            name: "PV".to_string(),
            technical_name: "pv".to_string(),
        },
        Sensor {
            name: "GRID".to_string(),
            technical_name: "grid".to_string(),
        }
    ];

    for sensor in sensors {
        let _ = publish_homeassistant_config_to_mqtt(
            &mut client,
            mqtt_homeassistant_discovery_topic.clone(),
            mqtt_homeassistant_state_topic.clone(),
            site_id.clone(),
            sensor
        ).await;
    }
}

fn generate_site_technical_name(
    site_id: String,
    technical_name: String,
) -> String {
    return "solaredge2mqtt".to_string()
        + &site_id.clone()
        + &technical_name.clone().replace("_", "");
}

fn generate_site_state_topic_name(
    mqtt_homeassistant_state_topic: String,
    site_id: String,
) -> String {
    return mqtt_homeassistant_state_topic.replace(
        "{}",
        &(PROJECT_NAME.to_string() + &site_id),
    );
}

async fn publish_homeassistant_config_to_mqtt(
    client: &mut AsyncClient,
    mqtt_homeassistant_discovery_topic: String,
    mqtt_homeassistant_state_topic: String,
    site_id: String,
    sensor: Sensor
) {
    let technical_name_without_dash = generate_site_technical_name(
        site_id.clone(),
        sensor.technical_name.clone(),
    );
    let config = json!(Config {
        device_class: HOMEASSISTANT_DEVICE_CLASS.to_string(),
        state_topic: generate_site_state_topic_name(mqtt_homeassistant_state_topic, site_id),
        unit_of_measurement: SOLAREDGE_UNIT.to_string(),
        value_template: ("{{ value_json.".to_string() + &sensor.technical_name + "}}").to_string(),
        unique_id: technical_name_without_dash.clone(),
        device: Device {
            identifiers: vec![
                technical_name_without_dash.clone()
            ],
            name: (PROJECT_NAME.to_string() + " " + &sensor.name).to_string(),
            manufacturer: PROJECT_NAME.to_string()
        }
    }).to_string();

    let _ = client.publish(
        mqtt_homeassistant_discovery_topic.replace("{}", &sensor.technical_name),
        QoS::AtLeastOnce,
        false,
        config,
    ).await.unwrap();
}

async fn publish_to_mqtt(
    client: &mut AsyncClient,
    result: SiteCurrentPowerFlow,
    site_id: String,
    mqtt_homeassistant_state_topic: String,
) {
    let channel = generate_site_state_topic_name(mqtt_homeassistant_state_topic, site_id);
    let payload = json!(generate_state_message(result)).to_string();
    log::debug!("Publish result to mqtt {}: {}", channel, payload);
    
    let _ = client.publish(
        channel,
        QoS::AtLeastOnce,
        false,
        payload,
    ).await.unwrap_or_else(|_| { panic!("Error during publishing") });
}

async fn update_from_solaredge(site_id: String, api_key: String) -> Result<SiteCurrentPowerFlow, String> 
{
    let base_url = format!("https://{SOLAREDGE_MONITORING_API_HOST}/site/{site_id}/currentPowerFlow?api_key={api_key}");
    let full_url = &base_url[..];
    let result = reqwest::get(full_url).await;
    if let Err(_) = result {
        return Err("Fail to fetch data from ".to_string() + SOLAREDGE_MONITORING_API_HOST);
    }
    let response = result.unwrap();
    return match response.status() {
        reqwest::StatusCode::OK => match response.json::<Root>().await {
            Ok(parsed) => Ok(parsed.site_current_power_flow),
            Err(_) => Err(SOLAREDGE_MONITORING_API_HOST.to_string() + " cannot be decoded"),
        },
        reqwest::StatusCode::UNAUTHORIZED => Err("Api key rejected for this site".to_string()),
        _ => Err("Fail to fetch data from ".to_string() + SOLAREDGE_MONITORING_API_HOST),
    };
}

fn generate_state_message(site_current_power_flow: SiteCurrentPowerFlow) -> State
{
    let mut result = State {
        pv_to_load: 0.0,
        load_to_grid: 0.0,
        grid_to_load: 0.0,
        pv: site_current_power_flow.pv.current_power,
        load: site_current_power_flow.load.current_power,
        grid: site_current_power_flow.grid.current_power,
    };

    if result.pv > 0.0 {
        result.pv_to_load = result.pv;
    } else {
        if result.pv == 0.0 {
            result.load_to_grid = 0.0;
            result.grid_to_load = result.load;
        } else if result.load < result.pv {
            result.load_to_grid = result.load - result.pv
        } else {
            result.grid_to_load = result.pv - result.load
        }
    }

    return result;
}
