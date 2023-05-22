use std::{collections::BTreeSet, fs::File, path::PathBuf};

use crate::error::{Error, InputFileParseSnafu};
use serde::{Deserialize, Deserializer, Serialize};
use snafu::ResultExt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Represents a particular type of hardware setup, like Intel+Nvidia Hybrid Graphics, or Nvidia Discrete Graphics, Intel+AMD Hybrid Graphics, etc.
pub struct HardwareCase {
    #[serde(default)]
    pub name: String,

    pub id: String,

    #[serde(default)]
    pub description: String,

    pub hardware_kind: HardwareKind,

    pub hardware_groups: BTreeSet<HardwareGroup>,

    pub driver_options: BTreeSet<DriverOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HardwareKind {
    #[serde(alias = "GRAPHICS", alias = "graphics")]
    Graphics,

    #[serde(alias = "ETHERNET", alias = "ethernet")]
    Ethernet,

    #[serde(alias = "WIRELESS", alias = "wireless")]
    Wireless,

    #[serde(alias = "SOUND", alias = "sound")]
    Sound,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Represents a group of Hardware that is catered by the same driver(s)
#[serde(transparent)]
pub struct HardwareGroup {
    pub device_entries: BTreeSet<DeviceEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Represents a group of devices that are entered together in the database for convenience
pub enum DeviceEntry {
    #[serde(alias = "PCI", alias = "pci")]
    Pci(PciIdList),

    #[serde(alias = "USB", alias = "usb")]
    Usb(UsbIdList),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PciIdList {
    #[serde(alias = "vendor-id", deserialize_with = "from_hex")]
    pub vendor: u16,

    #[serde(
        alias = "device-ids",
        alias = "device-id",
        deserialize_with = "from_hex_list"
    )]
    pub devices: BTreeSet<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UsbIdList {
    #[serde(alias = "vendor-id", deserialize_with = "from_hex")]
    pub vendor: u16,

    #[serde(
        alias = "device-ids",
        alias = "device-id",
        deserialize_with = "from_hex_list"
    )]
    pub devices: BTreeSet<u16>,
}

fn from_hex_list<'de, D>(deserializer: D) -> Result<BTreeSet<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: BTreeSet<&str> = Deserialize::deserialize(deserializer)?;
    s.iter()
        .map(|item| u16::from_str_radix(&item, 16).map_err(serde::de::Error::custom))
        .collect()
}

fn from_hex<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    u16::from_str_radix(&s, 16).map_err(serde::de::Error::custom)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DriverOption {
    #[serde(
        default,
        alias = "order-of-priority",
        alias = "order",
        alias = "priority"
    )]
    pub order_of_priority: u32,

    #[serde(default)]
    pub name: String,

    pub id: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub tags: BTreeSet<String>,

    #[serde(default)]
    pub packages: Vec<String>,

    #[serde(default)]
    pub pre_install: Option<Script>,

    #[serde(default)]
    pub post_install: Option<Script>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Script {
    pub path: PathBuf,
    pub language: ScriptKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScriptKind {
    #[serde(alias = "Py", alias = "py")]
    Python,

    #[serde(alias = "Js", alias = "js")]
    JavaScript,

    #[serde(alias = "Sh", alias = "sh")]
    Shell,
}

pub fn parse_input_file(path: PathBuf) -> Result<Vec<HardwareCase>, Error> {
    let file = File::open(&path).unwrap();
    Ok(serde_yaml::from_reader(&file).context(InputFileParseSnafu { path: path })?)
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    pub fn deserialize_input_data() {
        let f = File::open("input_data.yaml").unwrap();
        let deserialized_object: Vec<HardwareCase> = serde_yaml::from_reader(&f).unwrap();
        println!("The deserialized object... \n {:#?}", deserialized_object);
    }
}
