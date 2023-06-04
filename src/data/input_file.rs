use crate::error::Error;
use crate::error::InputFileParseSnafu;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize};
use snafu::ResultExt;
use std::str::FromStr;
use std::{collections::BTreeSet, fs::File, path::PathBuf};

use super::database::HardwareId;
use super::database::PciId;
use super::database::UsbId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Represents a particular type of hardware setup, like Intel+Nvidia Hybrid Graphics, or Nvidia Discrete Graphics, Intel+AMD Hybrid Graphics, etc.
pub struct HardwareSetup {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub description: String,

    pub hardware_kind: HardwareKind,

    pub hardware_list: HardwareList,

    pub driver_options: BTreeSet<DriverOption>,
}

impl HardwareSetup {
    pub fn matching_driver_options<'a, I>(
        &self,
        hardware_ids: &BTreeSet<HardwareId>,
        optional_hardware: &Option<HardwareKind>,
        mut tags: I,
    ) -> Option<BTreeSet<&DriverOption>>
    where
        I: Iterator<Item = &'a String>,
    {
        if let Some(hardware_kind) = optional_hardware {
            if &self.hardware_kind != hardware_kind {
                return None;
            }
        }
        if !self.hardware_list.matches_with_hardware_ids(hardware_ids) {
            return None;
        }
        return Some(
            self.driver_options
                .iter()
                .filter(|driver_option| tags.all(|tag| driver_option.tags.contains(tag)))
                .collect(),
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HardwareKind {
    #[serde(
        alias = "graphics",
        alias = "Graphics",
        alias = "GRAPHICS",
        alias = "gpu",
        alias = "Gpu",
        alias = "GPU"
    )]
    Graphics,

    #[serde(
        alias = "ethernet",
        alias = "Ethernet",
        alias = "ETHERNET",
        alias = "lan",
        alias = "Lan",
        alias = "LAN"
    )]
    Ethernet,

    #[serde(
        alias = "wireless",
        alias = "Wireless",
        alias = "WIRELESS",
        alias = "wifi",
        alias = "Wifi",
        alias = "WiFi",
        alias = "WIFI"
    )]
    Wireless,

    #[serde(
        alias = "sound",
        alias = "Sound",
        alias = "SOUND",
        alias = "audio",
        alias = "Audio",
        alias = "AUDIO"
    )]
    Audio,
}

impl FromStr for HardwareKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_string = String::from(s).to_lowercase();
        match user_string.as_str() {
            "graphics" | "Graphics" | "GRAPHICS" | "gpu" | "Gpu" | "GPU" => {
                Ok(HardwareKind::Graphics)
            }
            "ethernet" | "Ethernet" | "ETHERNET" | "lan" | "Lan" | "LAN" => {
                Ok(HardwareKind::Ethernet)
            }
            "wireless" | "Wireless" | "WIRELESS" | "wifi" | "Wifi" | "WiFi" | "WIFI" => {
                Ok(HardwareKind::Wireless)
            }
            "sound" | "Sound" | "SOUND" | "audio" | "Audio" | "AUDIO" => Ok(HardwareKind::Audio),
            _ => Err(Error::InvalidEnumValue {
                value: s.into(),
                enum_name: "HardwareKind".into(),
                allowed_values: vec![
                    "graphics", "Graphics", "GRAPHICS", "gpu", "Gpu", "GPU", "ethernet",
                    "Ethernet", "ETHERNET", "lan", "Lan", "LAN", "wireless", "Wireless",
                    "WIRELESS", "wifi", "Wifi", "WiFi", "WIFI", "sound", "Sound", "SOUND", "audio",
                    "Audio", "AUDIO",
                ]
                .into_iter()
                .map(|s| String::from(s))
                .collect(),
            }),
        }
    }
}

impl fmt::Display for HardwareKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HardwareKind::Graphics => write!(f, "graphics"),
            HardwareKind::Ethernet => write!(f, "ethernet"),
            HardwareKind::Wireless => write!(f, "wireless"),
            HardwareKind::Audio => write!(f, "audio"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HardwareList {
    #[serde(alias = "each")]
    /// Represents the presence of devices from each of the child groups
    Each(BTreeSet<HardwareListInner>),

    #[serde(alias = "PCI", alias = "pci")]
    Pci(PciIdList),

    #[serde(alias = "USB", alias = "usb")]
    Usb(UsbIdList),
}

impl HardwareList {
    pub fn matches_with_hardware_ids(&self, hardware_ids: &BTreeSet<HardwareId>) -> bool {
        return match self {
            HardwareList::Each(hardware_lists_inner) => {
                hardware_lists_inner.into_iter().all(|hardware_list_inner| {
                    return match hardware_list_inner {
                        HardwareListInner::Pci(pci_id_list) => {
                            pci_id_list.devices.iter().any(|device| {
                                hardware_ids.contains(&HardwareId::Pci(PciId {
                                    vendor: pci_id_list.vendor,
                                    device: *device,
                                }))
                            })
                        }
                        HardwareListInner::Usb(usb_id_list) => {
                            usb_id_list.devices.iter().any(|device| {
                                hardware_ids.contains(&HardwareId::Usb(UsbId {
                                    vendor: usb_id_list.vendor,
                                    device: *device,
                                }))
                            })
                        }
                    };
                })
            }
            HardwareList::Pci(pci_id_list) => pci_id_list.devices.iter().any(|device| {
                hardware_ids.contains(&HardwareId::Pci(PciId {
                    vendor: pci_id_list.vendor,
                    device: *device,
                }))
            }),
            HardwareList::Usb(usb_id_list) => usb_id_list.devices.iter().any(|device| {
                hardware_ids.contains(&HardwareId::Usb(UsbId {
                    vendor: usb_id_list.vendor,
                    device: *device,
                }))
            }),
        };
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HardwareListInner {
    #[serde(alias = "PCI", alias = "pci")]
    Pci(PciIdList),

    #[serde(alias = "USB", alias = "usb")]
    Usb(UsbIdList),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PciIdList {
    #[serde(alias = "vendor-id", alias = "vendor", deserialize_with = "from_hex")]
    pub vendor: u16,

    #[serde(
        alias = "device-ids",
        alias = "device-id",
        alias = "devices",
        alias = "device",
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
    s.into_iter()
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
        alias = "priority",
        alias = "rank",
        alias = "ranking"
    )]
    pub order_of_priority: u32,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub tags: BTreeSet<String>,

    #[serde(default, alias = "pre-install", alias = "preinstall")]
    pub pre_install: Option<Script>,

    #[serde(default)]
    pub packages: Vec<String>,

    #[serde(default, alias = "post-install", alias = "postinstall")]
    pub post_install: Option<Script>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Script {
    pub path: PathBuf,
    pub language: ScriptKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScriptKind {
    #[serde(alias = "PY", alias = "Py", alias = "py")]
    Python,

    #[serde(
        alias = "JS",
        alias = "Js",
        alias = "js",
        alias = "node",
        alias = "nodejs",
        alias = "node-js",
        alias = "node_js"
    )]
    JavaScript,

    #[serde(alias = "SH", alias = "Sh", alias = "sh")]
    Shell,
}

pub fn parse_input_file(path: PathBuf) -> Result<BTreeSet<HardwareSetup>, Error> {
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
        let deserialized_object: Vec<HardwareSetup> = serde_yaml::from_reader(&f).unwrap();
        println!("The deserialized object... \n {:#?}", deserialized_object);
    }
}
