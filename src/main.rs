use esp_idf_svc::hal::adc::AdcDriver;

use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::mqtt::client::{EventPayload, QoS};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::info;
use std::str::from_utf8;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use crate::mqtt::publish_to_topic;

//Custom modules
pub mod wifi;
pub mod led;
pub mod mqtt;
pub mod nvs;

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
    mqtt_topic_sub: &'static str,
    #[default("")]
    mqtt_topic_pub: &'static str,
    #[default("")]
    mqtt_client_id: &'static str,
}

static THRESHOLD: AtomicU32 = AtomicU32::new(10);



fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    let app_config = CONFIG;

    /* Configure ESP peripherals */
    let peripherals = Peripherals::take().unwrap();

    let mut adc1 = AdcDriver::new(peripherals.adc1, &esp_idf_svc::hal::adc::config::Config::new().calibration(true)).unwrap();
    let mut a1_ch0 =
        esp_idf_svc::hal::adc::AdcChannelDriver::<{ esp_idf_svc::hal::adc::attenuation::DB_11 }, _>::new(peripherals.pins.gpio0)
            .unwrap();
    

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs_default_partition = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs_default_partition.clone())).unwrap(),
        sys_loop,
    ).unwrap();
    let mut led = led::AddressableLed::new(peripherals.pins.gpio8, peripherals.rmt.channel0).unwrap();
    
    led::led_off(&mut led).unwrap(); 
    /* Configure WiFi */
    let configuration = wifi::configure_wifi(app_config.wifi_ssid, app_config.wifi_pass).unwrap();

    match wifi::connect_wifi(&mut wifi, &configuration){
        Ok(_)=>{
            led::led_green(&mut led).unwrap();
        }
        Err(err)=>{
            led::led_red(&mut led).unwrap();
            eprintln!("Error: {:?}", err);
        }
    }

    /* Load the certificate */
    let ca_cert_bytes = include_bytes!("../certificates/ca_cert.pem");
    /* Key in the nvs to associate the certificate with */
    let key = "ca_cert";
    /* Create namespace */
    let mut nvs = nvs::create_certificate_namespace(nvs_default_partition, "cert_namespace").unwrap();
    /* Save the certificate to the nvs */
    nvs::save_cert(&mut nvs, key,ca_cert_bytes);
    /* Retrieve the certificate from the nvs */
    let cert_from_nvs = nvs::retrieve_cert(&mut nvs, key, ca_cert_bytes);
    /* Convert the certificate to the correct format */
    let ca_cert = mqtt::convert_certificate(cert_from_nvs);

    /* Configure MQTT */
    let (mut mqtt_client, mut mqtt_conn) = mqtt::mqtt_create_client_connection(app_config.mqtt_url, ca_cert, app_config.mqtt_client_id, app_config.mqtt_user, app_config.mqtt_pass).unwrap();
    
    /* Thread to run the mqtt listener */
    thread::spawn( move||{
        println!("Received signal to start mqtt listener");
        while let Ok(event) = mqtt_conn.next() {
            let a=event.payload();
            match a{
                EventPayload::Received { id, topic, data, details }=> {
                    match topic{
                        Some(x)=>{
                            if x == "threshold"{
                                let msg: u32 = from_utf8(data).unwrap().parse().unwrap();
                                THRESHOLD.store(msg,Ordering::Relaxed);
                                info!("received new threshold: {:?}",msg);
                            }
                        },
                        None=>()
                    }
                },
                _ => ()
                }
            }
    });

    /* Subscribe to the topic */
    mqtt::subscribe_to_topic(&mut mqtt_client, app_config.mqtt_topic_sub).unwrap_or_default();
    let mut state=false; // too hot=>true, else => false
    const B: f64 = 3950.0; // B value of the thermistor
    loop{
        match wifi::check_wifi_connection(&mut wifi){
            Ok(true)=>{
                led::led_green(&mut led).unwrap();
                let threshold=THRESHOLD.load(Ordering::Relaxed);
                let mut payload:u32 = 1;
                match adc1.read(&mut a1_ch0) {
                    Ok(sample) => {
                        if sample>0{
                        let temperature=1. / ((1. / (4096. / sample as f64 - 1.)).ln() / B + 1.0 / 298.15) - 273.15;
                        println!("A1_CH0: {temperature}\n");
                        println!("sample: {sample}\n");
                        payload=temperature as u32;
                        state=mqtt::publish_to_topic(&mut mqtt_client, app_config.mqtt_topic_pub, payload,threshold,state).unwrap();
                    }
                        },
                    Err(e) => println!("err reading A1_CH0: {e}\n"),
                }
                //state=mqtt::publish_to_topic(&mut mqtt_client, app_config.mqtt_topic, payload,threshold,state).unwrap();
                
            }
            Ok(false)=>{
                led::led_red(&mut led).unwrap();
                panic!("Wifi connection lost");
            }
            Err(err)=>{
                led::led_red(&mut led).unwrap();
                eprintln!("Error: {:?}", err);
                break;
            } 
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}


