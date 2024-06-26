use esp_idf_svc::nvs::*;
use esp_idf_svc::nvs::NvsDefault;
use esp_idf_svc::nvs::EspNvsPartition;
use esp_idf_svc::sys::EspError;
use log::info;

///Create the certificate namespace to save the certificate to.
/// # Arguments
/// * `nvs_default_partition` - The NVS partition to create the namespace in.
/// * `certificate_namespace` - The namespace to create.
/// # Returns
/// The NVS partition with the certificate namespace.
/// # Errors
/// Returns an error if the namespace fails to be created.
pub fn create_certificate_namespace(nvs_default_partition: EspNvsPartition<NvsDefault>, certificate_namespace: &str)-> Result<EspNvs<NvsDefault>,EspError>  {
    let nvs = match EspNvs::new(nvs_default_partition, certificate_namespace, true) {
        Ok(nvs) => {
            info!("Got namespace {:?} from default partition", certificate_namespace);
            nvs
        }
        Err(e) => panic!("Couldn't get namespace {:?}", e),
    };
    Ok(nvs)
}

/// Saves the certificate to the NVS partition.
/// # Arguments
/// * `nvs` - The NVS partition to save the certificate to.
/// * `key` - The key to save the certificate under.
/// * `certificate_data` - The certificate data to save.
pub fn save_cert(nvs: &mut EspNvs<NvsDefault>,key: &str ,certificate_data: &[u8]){
    match nvs.set_raw(key, certificate_data) {
        Ok(_) => info!("Key {} updated", key),
        Err(e) => info!("Key {} not updated {:?}", key, e),
    };
}


/// Retrieves the certificate from the NVS partition.
/// # Arguments
/// * `nvs` - The NVS partition to retrieve the certificate from.
/// * `key` - The key to retrieve the certificate from.
/// * `certificate_data` - The certificate data to retrieve.
/// # Returns
/// The retrieved certificate data.
pub fn retrieve_cert(nvs: &mut EspNvs<NvsDefault>, key: &str, certificate_data: &[u8]) -> Vec<u8> {
    let mut read_data = vec![0u8; certificate_data.len()];
    match nvs.get_raw(key, &mut read_data) {
        Ok(v) => {
            if let Some(the_data) = v {
                // Do something with the read data
                let nvs_data = the_data;
                nvs_data.to_vec() // Return the retrieved data
            } else {
                panic!("No data found for key {}", key); // Panic if no data is found for the key
            }
        }
        Err(e) => {
            panic!("Couldn't get key {} because {:?}", key, e); // Panic in case of error
        }
    }
}