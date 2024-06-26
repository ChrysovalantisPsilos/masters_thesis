use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use anyhow::{Context, Error};

/// Configures the WiFi connection with the provided SSID and password.
/// Returns the WiFi configuration.
/// # Arguments
/// * `ssid` - The SSID of the WiFi network.
/// * `password` - The password of the WiFi network.
/// # Returns
/// The WiFi configuration.
/// # Errors
/// Returns an error if the SSID or password is invalid.
pub fn configure_wifi(ssid: &str, password: &str) -> Result<Configuration,Error> {
    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap_or_default(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.try_into().unwrap_or_default(),
        channel: None,
    });
    Ok(wifi_configuration)
}

/// Connects to the WiFi network using the provided WiFi configuration.
/// # Arguments
/// * `wifi` - The WiFi instance.
/// * `config` - The WiFi configuration.
/// # Returns
/// The Ok result 0 if the connection is successful.
/// # Errors
/// Returns an error if the connection fails.
pub fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>, config: &Configuration) -> Result<(),Error> {
    if check_wifi_connection(wifi)?{
        return Ok(())
    }else{
        wifi.set_configuration(&config).context("failed to set the configuration")?;
        wifi.start().context("Failed to start the WiFi")?;        
        wifi.connect().context("Failed to connect. Make sure you have valid credentials in the .cfg file")?;
        wifi.wait_netif_up()?;
        Ok(())
    }
}

/// Disconnects from the WiFi network.
/// # Arguments
/// * `wifi` - The WiFi instance.
/// # Returns
/// The Ok result 0 if the disconnection is successful.
/// # Errors
/// Returns an error if the disconnection fails.
pub fn disconnect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<i32,Error> {
    wifi.disconnect().context("Failed to disconnect WiFi")?;
    Ok(0)
}

/// Checks if the WiFi is connected.
/// # Arguments
/// * `wifi` - The WiFi instance.
/// # Returns
/// Returns `true` if the WiFi is connected.
/// Returns `false` if the WiFi is not connected.
pub fn check_wifi_connection(wifi:&mut BlockingWifi<EspWifi<'static>>)-> Result<bool,Error> {
    if wifi.is_connected().unwrap(){
        Ok(true)
    }
    else{
        Ok(false)
    }
 }
