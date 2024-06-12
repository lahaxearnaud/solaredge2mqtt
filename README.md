# Solaredge2MQTT

Solaredge2MQTT is a Rust application that fetches energy production data from your SolarEdge system and publishes it to an MQTT broker. This application is designed to easily integrate with Home Assistant via the MQTT auto-discovery mechanism.

## Prerequisites

- Rust and Cargo installed on your machine.
- A SolarEdge account with API access and a configured site.
- An MQTT broker (like Mosquitto).
- Home Assistant with MQTT auto-discovery enabled.

## Installation

1. Clone the repository:

    ```sh
    git clone https://github.com/lahaxearnaud/solaredge2mqtt.git
    cd solaredge2mqtt
    ```

2. Configure the environment variables by copying the `.env.dist` file and renaming it to `.env`, then filling in the appropriate values:

    ```sh
    cp .env.dist .env
    ```

    Then, edit the `.env` file to include your own information:

    ```sh
    SITE_ID=000000
    API_KEY=YOUR_API_KEY
    DELAY_BETWEEN_UPDATE_S=500
    MQTT_CLIENT_NAME=solaredge2mqtt
    MQTT_HOST=YOUR_MQTT_HOST
    MQTT_PORT=1883
    MQTT_USERNAME=YOUR_MQTT_USERNAME
    MQTT_PASSWORD=YOUR_MQTT_PASSWORD
    MQTT_HOMEASSISTANT_DISCOVERY_TOPIC=homeassistant/sensor/solaredge2mqtt{}/config
    MQTT_HOMEASSISTANT_STATE_TOPIC=homeassistant/sensor/solaredge2mqtt{}/state
    ```

## Usage

### Starting the Application

To start the application, you can use the following commands:

- To run the application in development mode:

    ```sh
    make start
    ```

- To watch for changes and automatically recompile:

    ```sh
    make watch
    ```

- To build the application in release mode:

    ```sh
    make build
    ```

### Using Docker

You can also build and run the application using Docker.

- To build the Docker image:

    ```sh
    make build-docker
    ```

- To publish the Docker image:

    ```sh
    make publish-docker
    ```

- To run the Docker container:

    ```sh
    docker run --env-file .env lahaxearnaud/solaredge2mqtt
    ```

## Integration with Home Assistant

1. Ensure that MQTT auto-discovery is enabled in Home Assistant.
2. Configure the discovery and state topics in the `.env` file.
3. Start the application as described above.

Home Assistant should automatically discover the sensors published by Solaredge2MQTT.


## Contribution

Contributions are welcome! If you have suggestions, bugs to report, or features to add, feel free to open an issue or a pull request.

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.

---

Thank you for using Solaredge2MQTT! If you have any questions or issues, feel free to open an issue on GitHub.
