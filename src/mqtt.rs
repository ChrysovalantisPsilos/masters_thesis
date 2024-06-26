use anyhow::Error;
use log::info;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::tls::X509;
use std::{mem, slice};

const HYSTERESIS: u32 =2;

/// Reads the certificate file and converts it to the correct format.
/// # Arguments
/// * `certificate_bytes` - The certificate file in bytes.
/// # Returns
/// The certificate file in the correct format.
pub fn convert_certificate(mut certificate_bytes: Vec<u8>) -> X509<'static> {
    
    // append NUL-->certificate needs to be null-terminated
    certificate_bytes.push(0);

    // convert the certificate
    let certificate_slice: &[u8]= unsafe {
        let ptr: *const u8 = certificate_bytes.as_ptr();
        let len: usize = certificate_bytes.len();
        mem::forget(certificate_bytes);
        slice::from_raw_parts(ptr, len)
    };

    // return the certificate file in the correct format
    X509::pem_until_nul(certificate_slice)
}


/// Creates a new MQTT client and establishes a connection.
/// # Arguments
/// * `url` - The URL of the MQTT broker.
/// * `server_cert` - The server certificate to use for the MQTT connection.
/// * `client_id` - The client ID to use for the MQTT connection.
/// * `username` - The username to use for the MQTT connection.
/// * `password` - The password to use for the MQTT connection.
/// # Returns
/// A tuple containing the MQTT client and the MQTT connection.
/// # Errors
/// Returns an error if the MQTT client fails to initialize or establish a connection.
pub fn mqtt_create_client_connection(url: &str, server_cert: X509<'static>, client_id: &str, username: &'static str, password: &'static str) -> Result<(EspMqttClient<'static>, EspMqttConnection), EspError> {
    let (mqtt_client, mqtt_conn) = EspMqttClient::new(
        url,
        &MqttClientConfiguration {
            client_id: Some(client_id),
            username: Some(username),
            password: Some(password),
            server_certificate: Some(server_cert),
            ..Default::default()
        },
    )?;
    info!("MQTT client created");
    Ok((mqtt_client, mqtt_conn))
}


/// Starts the MQTT listener.
/// # Arguments
/// * `connection` - The MQTT connection to listen on.
/// # Returns
/// Returns `Ok` if the listener is started successfully.
/// Returns an error if the listener fails to start.
pub fn start_mqtt_listener(connection: &mut EspMqttConnection) -> Result<(),Error>{    
    while let Ok(event) = connection.next() {
        info!("[Queue] Event: {:?}", event.payload());
    }
    Ok(())
}

/// Subscribes to a topic.
/// # Arguments
/// * `client` - The MQTT client to use.
/// * `topic` - The topic to subscribe to.
/// # Returns
/// Returns `Ok` if the subscription is successful.
/// Returns an error if the subscription fails.
pub fn subscribe_to_topic(client: &mut EspMqttClient<'_>, topic: &str) -> Result<(), EspError> {
    client.subscribe(topic, QoS::AtMostOnce)?;
    info!("Subscribed to topic \"{topic}\"");
    Ok(())
}

/// Publishes a payload to a topic.
/// # Arguments
/// * `client` - The MQTT client to use.
/// * `topic` - The topic to publish to.
/// * `payload` - The payload to publish.
/// * `threshold` - The threshold to compare the payload against.
/// # Returns
/// Returns `Ok` if the payload is published successfully.
/// Returns an error if the payload fails to publish.
pub fn publish_to_topic(client: &mut EspMqttClient<'_>, topic: &str, payload: u32, threshold: u32,state: bool) -> Result<bool, EspError> {
    let mut new_state: bool=false;
    match state{
        false =>{
            if payload>threshold+HYSTERESIS{
                new_state=true;
                let payload_in_bytes=payload.to_be_bytes();
                client.enqueue(topic, QoS::AtMostOnce, false, &payload_in_bytes).unwrap();
                info!("Published {} to topic {}",payload,topic);
            }
            else{
                info!("Payload is ({}) less than threshold, not publishing",payload);
            }
        }
        true =>{
            if payload<threshold-HYSTERESIS{
                info!("Payload is ({}) less than threshold, not publishing",payload);
            }
            else{
                new_state=true;
                let payload_in_bytes=payload.to_be_bytes();
                client.enqueue(topic, QoS::AtMostOnce, false, &payload_in_bytes).unwrap();
                info!("Published {} to topic {}",payload,topic);
            }
        }
    }
    Ok(new_state)
}




