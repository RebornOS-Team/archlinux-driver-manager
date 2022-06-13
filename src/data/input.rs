use serde::{Serialize, Deserialize};
use serde_aux::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DriverEntry {
    name: String,
    description: String,
    tags: Vec<String>,
    #[serde(deserialize_with = "deserialize_struct_case_insensitive")]
    ids: Vec<HardwareIdEntry>,
    packages: Vec<String>,
    configurations: Vec<Configuration>,
    pre_install: Script,
    post_install: Script,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum HardwareIdEntry {
    Pci(PciIdList),
    Usb(UsbIdList),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PciIdList {
    vendor: u16,
    devices: Vec<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UsbIdList {
    vendor: u16,
    devices: Vec<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Configuration {
    format: String,
    path: String,
    entries: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Script {
    path: String,
    language: String,
}

#[cfg(test)]
mod tests {
    use std::{fs::File};

    use super::DriverEntry;

    #[test]
    pub fn deserialize_input_data() {
        let f = File::open("input_data.yaml").unwrap();
        let deserialized_drivers: Vec<DriverEntry> = serde_yaml::from_reader(f).unwrap();
        println!("{:#?}", deserialized_drivers);
    }
}
