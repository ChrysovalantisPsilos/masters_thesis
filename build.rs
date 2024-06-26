// main.rs or build.rs
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_pass: &'static str,
    #[default("")]
    mqtt_url: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default("")]
    mqtt_topic: &'static str,
    #[default("")]
    mqtt_client_id: &'static str,
}

fn main() {
    let app_config = CONFIG;

    if app_config.wifi_pass == "your_pass" || app_config.wifi_ssid == "your_ssid" || app_config.mqtt_url == "your_mqtt_url" || app_config.mqtt_user == "your_mqtt_user" || app_config.mqtt_pass == "your_mqtt_pass" || app_config.mqtt_topic == "your_mqtt_topic" || app_config.mqtt_client_id == "your_mqtt_client_id" {
        panic!("Please set your credentials in the .cfg file");
    }
    embuild::espidf::sysenv::output();
}
