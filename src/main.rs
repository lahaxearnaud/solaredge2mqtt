mod solaredge;
mod homeassistant;
mod mqtt;

extern crate dotenv;

use mqtt::state::State;
use homeassistant::config::{Config, Device};

use std::env;
use solaredge::current_power_flow::{Root, SiteCurrentPowerFlow};
use dotenv::dotenv;
use simple_logger::SimpleLogger;
use reqwest;
use tokio::time::Duration;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json::json;

#[tokio::main]
async fn main() {
    dotenv().ok();
    SimpleLogger::new().init().unwrap();

    let update_delay = env::var("DELAY_BETWEEN_UPDATE_S")
        .unwrap_or_else(|_| { panic!("API_KEY env var is missing") })
        .to_string()
        .parse::<u64>()
        .unwrap();
    let api_key = env::var("API_KEY")
        .unwrap_or_else(|_| { panic!("API_KEY env var is missing") });
    let site_id = env::var("SITE_ID")
        .unwrap_or_else(|_| { panic!("SITE_ID env var is missing") });
    let mqtt_client_name = env::var("MQTT_CLIENT_NAME")
        .unwrap_or("solaredge2mqtt".to_string());
    let mqtt_host = env::var("MQTT_HOST")
        .unwrap_or_else(|_| { panic!("MQTT_HOST env var is missing") });
    let mqtt_port = env::var("MQTT_PORT")
        .unwrap_or_else(|_| { panic!("MQTT_PORT env var is missing") })
        .parse::<u16>()
        .unwrap();
    let mqtt_username = env::var("MQTT_USERNAME")
        .unwrap_or_else(|_| { panic!("MQTT_USERNAME env var is missing") });
    let mqtt_password = env::var("MQTT_PASSWORD")
        .unwrap_or_else(|_| { panic!("MQTT_PASSWORD env var is missing") });

    let mqtt_homeassistant_discovery_topic = env::var("MQTT_HOMEASSISTANT_DISCOVERY_TOPIC")
        .unwrap_or_else(|_| { panic!("MQTT_HOMEASSISTANT_DISCOVERY_TOPIC env var is missing") });
    let mqtt_homeassistant_state_topic = env::var("MQTT_HOMEASSISTANT_STATE_TOPIC")
        .unwrap_or_else(|_| { panic!("MQTT_HOMEASSISTANT_STATE_TOPIC env var is missing") });

    let mut mqttoptions = MqttOptions::new(mqtt_client_name, mqtt_host, mqtt_port);
    mqttoptions.set_credentials(mqtt_username, mqtt_password);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let _ = publish_homeassistant_config_to_mqtt(
        &mut client,
        mqtt_homeassistant_discovery_topic.clone(),
        mqtt_homeassistant_state_topic.clone(),
        site_id.clone(),
        "pv_to_load".to_string(),
        "PV to LOAD".to_string(),

    ).await;

    let _ = publish_homeassistant_config_to_mqtt(
        &mut client,
        mqtt_homeassistant_discovery_topic.clone(),
        mqtt_homeassistant_state_topic.clone(),
        site_id.clone(),
        "load_to_grid".to_string(),
        "LOAD to GRID".to_string(),
    ).await;

    let _ = publish_homeassistant_config_to_mqtt(
        &mut client,
        mqtt_homeassistant_discovery_topic.clone(),
        mqtt_homeassistant_state_topic.clone(),
        site_id.clone(),
        "grid_to_load".to_string(),
        "GRID to LOAD".to_string(),
    ).await;

    let _ = publish_homeassistant_config_to_mqtt(
        &mut client,
        mqtt_homeassistant_discovery_topic.clone(),
        mqtt_homeassistant_state_topic.clone(),
        site_id.clone(),
        "pv".to_string(),
        "PV".to_string(),
    ).await;

    let _ = publish_homeassistant_config_to_mqtt(
        &mut client,
        mqtt_homeassistant_discovery_topic.clone(),
        mqtt_homeassistant_state_topic.clone(),
        site_id.clone(),
        "load".to_string(),
        "LOAD".to_string(),
    ).await;

    let _ = publish_homeassistant_config_to_mqtt(
        &mut client,
        mqtt_homeassistant_discovery_topic.clone(),
        mqtt_homeassistant_state_topic.clone(),
        site_id.clone(),
        "grid".to_string(),
        "GRID".to_string(),
    ).await;

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    log::debug!("Notification: {:?}", notification)
                },
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
                api_key.clone()
            ).await.unwrap();

           let _ = publish_to_mqtt(
                &mut client,
                site_current_power_flow,
                site_id.clone(),
                mqtt_homeassistant_state_topic.clone()
            ).await;
       }
   }).await;
}

fn generate_site_technical_name(
    site_id: String,
    technical_name: String
) -> String {
    return "solaredge2mqtt".to_string()
        + site_id.clone().as_str()
        + technical_name.clone().replace("_", "").as_str()
}

fn generate_site_state_topic_name(
    mqtt_homeassistant_state_topic: String,
    site_id: String
) -> String {
    return mqtt_homeassistant_state_topic.replace(
        "{}",
        ("solaedge2mqtt".to_string() + site_id.as_str()).as_str()
    )
}

async fn publish_homeassistant_config_to_mqtt(
    client: &mut AsyncClient,
    mqtt_homeassistant_discovery_topic: String,
    mqtt_homeassistant_state_topic: String,
    site_id: String,
    technical_name: String,
    friendly_name: String
) {
    let technical_name_without_dash = generate_site_technical_name(
        site_id.clone(),
        technical_name.clone()
    );
    let config = json!(Config {
        device_class: "power".to_string(),
        state_topic: generate_site_state_topic_name(mqtt_homeassistant_state_topic, site_id),
        unit_of_measurement: "kW".to_string(),
        value_template: ("{{ value_json.".to_string() + &technical_name + "}}").to_string(),
        unique_id: technical_name_without_dash.clone(),
        device: Device {
            identifiers: vec![
                technical_name_without_dash.clone()
            ],
            name: ("Solaredge2mqtt ".to_string() + &friendly_name).to_string(),
            manufacturer: "Solaredge".to_string()
        }
    }).to_string();
    let _ = client.publish(
        mqtt_homeassistant_discovery_topic.replace("{}", &technical_name),
        QoS::AtLeastOnce,
        false,
        config
    ).await.unwrap_or_else(|_| { panic!("Error during publish") });
}


async fn publish_to_mqtt(
    client: &mut AsyncClient,
    result: SiteCurrentPowerFlow,
    site_id: String,
    mqtt_homeassistant_state_topic: String
) {
    log::debug!("Publish result to mqtt {}", "hello/rumqtt");

    let paylaod = json!(generate_state_message(result)).to_string();
    println!("{}", paylaod);

    let _ = client.publish(
        generate_site_state_topic_name(mqtt_homeassistant_state_topic, site_id),
        QoS::AtLeastOnce,
        false,
        paylaod
    ).await.unwrap_or_else(|_| { panic!("Error during publish") });
}

async fn update_from_solaredge(site_id: String, api_key: String) -> Result<SiteCurrentPowerFlow, &'static str> {
    let base_url = format!("https://monitoringapi.solaredge.com/site/{}/currentPowerFlow?api_key={}", site_id, api_key);
    let full_url = &base_url[..];
    let result = reqwest::get(full_url).await;
    let response = result.unwrap();
    return match response.status() {
        reqwest::StatusCode::OK => match response.json::<Root>().await {
            Ok(parsed) => Ok(parsed.site_current_power_flow),
            Err(_) => Err("Fail to fetch data from monitoringapi.solaredge.com"),
        },
        reqwest::StatusCode::UNAUTHORIZED => Err("Api key rejected for this site"),
        _ => Err("Fail to fetch data from monitoringapi.solaredge.com"),
    };
}

fn generate_state_message(site_current_power_flow: SiteCurrentPowerFlow) -> State
{
    let mut result = State{
        pv_to_load: 0.0,
        load_to_grid: 0.0,
        grid_to_load: 0.0,
        pv: site_current_power_flow.pv.current_power,
        load: site_current_power_flow.load.current_power,
        grid: site_current_power_flow.grid.current_power
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
