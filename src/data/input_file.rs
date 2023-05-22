use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    path::PathBuf,
};

use crate::error::{Error, InputFileParseSnafu};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DriverEntry {
    #[serde(
        default,
        alias = "order-of-priority",
        alias = "order",
        alias = "priority"
    )]
    pub order_of_priority: u32,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub description: String,

    pub hardware_kind: HardwareKind,

    #[serde(default)]
    pub tags: BTreeSet<String>,

    #[serde(default)]
    pub ids: Vec<HardwareIdEntry>,

    #[serde(default)]
    pub packages: Vec<String>,

    #[serde(default, alias = "configs")]
    pub configurations: Vec<Configuration>,

    #[serde(default)]
    pub pre_install: Option<Script>,

    #[serde(default)]
    pub post_install: Option<Script>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HardwareIdEntry {
    #[serde(alias = "PCI", alias = "pci")]
    Pci(PciIdList),

    #[serde(alias = "USB", alias = "usb")]
    Usb(UsbIdList),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PciIdList {
    pub vendor: String,
    pub devices: Vec<String>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct UsbIdList {
    pub vendor: String,
    pub devices: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    pub path: PathBuf,
    pub language: ScriptKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub format: ConfigurationFormat,
    pub path: PathBuf,
    pub entry_map: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigurationFormat {
    #[serde(alias = "INI", alias = "ini")]
    Ini,

    #[serde(alias = "JSON", alias = "json")]
    Json,

    #[serde(alias = "YAML", alias = "yaml")]
    Yaml,

    #[serde(alias = "TOML", alias = "toml")]
    Toml,

    #[serde(alias = "XML", alias = "xml")]
    Xml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptKind {
    #[serde(alias = "Py", alias = "py")]
    Python,

    #[serde(alias = "Js", alias = "js")]
    JavaScript,

    #[serde(alias = "Sh", alias = "sh")]
    Shell,
}

impl Default for HardwareKind {
    fn default() -> Self {
        HardwareKind::Graphics
    }
}

impl Default for HardwareIdEntry {
    fn default() -> Self {
        return HardwareIdEntry::Pci(PciIdList::default());
    }
}

pub fn parse_input_file(path: PathBuf) -> Result<Vec<DriverEntry>, Error> {
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
        let deserialized_object: Vec<DriverEntry> = serde_yaml::from_reader(&f).unwrap();
        println!("The deserialized object... \n {:#?}", deserialized_object);
    }

    #[test]
    pub fn serialize_deserialize_input_data() {
        let driver_entry_1 = DriverEntry {
            order_of_priority: 1,
            name: "Nvidia".to_string(),
            description: "Graphics driver for Nvidia GPU from the `nvidia` package found in the official Arch Linux `Extra` repository.".to_string(),
            hardware_kind: HardwareKind::Graphics,
            tags: vec![
                "proprietary".to_string(),
                "closed source".to_string(),
                "non free".to_string()
            ].into_iter().collect(),
            ids: vec![
                HardwareIdEntry::Pci (
                    PciIdList {
                        vendor: "10de".to_string(),
                        devices: vec!["1381".to_string(), "1392".to_string()],
                    }
                )
            ],
            packages: vec!["nvidia".to_string()],
            configurations: vec![
                Configuration {
                    format: ConfigurationFormat::Ini,
                    path: "~/temp/myconf.conf".into(),
                    entry_map: {
                        let mut entry_map = HashMap::new();
                        entry_map.insert("Hello".to_string(), "World".to_string());
                        entry_map
                    },
                }
            ],
            pre_install: Some(Script {
                path: "dummy_pre.py".into(),
                language: ScriptKind::Python,
            }),
            post_install: Some(Script {
                path: "dummy_post.sh".into(),
                language: ScriptKind::Shell,
            }),
        };
        let serialized_string = serde_yaml::to_string(&driver_entry_1).unwrap();
        println!("The serialized string... \n {}", serialized_string);

        let deserialized_object: DriverEntry = serde_yaml::from_str(&serialized_string).unwrap();
        println!("The deserialized object... \n {:#?}", deserialized_object);
    }
}
