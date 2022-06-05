use rangemap::{RangeInclusiveMap, StepLite};
use rustbreak::{deser::Ron, FileDatabase};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::num::ParseIntError;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops::{Range, RangeInclusive},
    path::PathBuf,
    str::FromStr,
};

// pub type DriverListing = RangeInclusiveMap<u32, Vec<DriverRecord>>;
pub type DriverListing = RangeInclusiveMap<PciId, Vec<DriverRecord>>;
pub type DriverDatabase = FileDatabase<DriverListing, Ron>;

// region: DATA MODEL

#[derive(
    Default,
    PartialEq,
    Eq,
    PartialOrd, // Required by Ord
    Ord,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Copy,
    Clone, // Required by RangeInclusiveMap to implement Serialize and Deserialize
)]
pub struct PciId {
    value: u32,
}

#[derive(Clone, Debug)]
pub enum ParsePciIdError {
    InvalidVendorId(ParseIntError),
    InvalidDeviceId(ParseIntError),
    MissingColon,
}

#[derive(
    Default,
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Serialize,
    Deserialize,
)]
pub struct DriverRecord {
    pub packages: Vec<String>,
    pub configs: Vec<ConfigRecord>,
    pub tags: Vec<String>,
    pub pre_install_script: Option<Script>,
    pub post_install_script: Option<Script>,
}

#[derive(
    Default,
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Serialize,
    Deserialize,
)]
pub struct ConfigRecord {
    pub format: ConfigFormat,
    pub path: Option<PathBuf>,
    pub entries: HashMap<String, String>,
}

#[derive(
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Serialize,
    Deserialize,
)]
pub enum ConfigFormat {
    Ini,
    Json,
    Yaml,
    Toml,
    Xml,
}

#[derive(
    Default,
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Serialize,
    Deserialize,
)]
pub struct Script {
    pub script_kind: ScriptKind,
    pub path: Option<PathBuf>,
}

#[derive(
    Debug,
    PartialEq, // Required to implement Eq
    Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
    Serialize,
    Deserialize,
)]
pub enum ScriptKind {
    Python,
    JavaScript,
    Shell,
}

// endregion: DATA MODEL

// region: IMPLEMENTATIONS

impl PciId {
    pub fn new(vendor_id: u16, device_id: u16) -> Self {
        Self {
            value: (vendor_id as u32) * 16u32.pow(4) + (device_id as u32),
        }
    }

    pub fn vendor_id(&self) -> u16 {
        let vendor_id = self.value / 16u32.pow(4);
        println!(
            "self.value: {:08x}, vendor_id: {:04x}",
            self.value, vendor_id
        );
        vendor_id
            .try_into()
            .expect("The Vendor ID does not fit into an unsigned 16-bit integer.")
    }

    pub fn device_id(&self) -> u16 {
        let device_id = self.value % 16u32.pow(4);
        println!(
            "self.value: {:08x}, device_id: {:04x}",
            self.value, device_id
        );
        device_id
            .try_into()
            .expect("The Device ID does not fit into an unsigned 16-bit integer.")
    }

    pub fn range(start: &str, end: &str) -> Result<Range<Self>, ParsePciIdError> {
        Ok(Range {
            start: start.parse()?,
            end: end.parse()?,
        })
    }

    pub fn range_inclusive(
        start: &str,
        end: &str,
    ) -> Result<RangeInclusive<Self>, ParsePciIdError> {
        Ok(RangeInclusive::new(start.parse()?, end.parse()?))
    }
}

impl Display for PciId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}:{:04x}", self.vendor_id(), self.device_id())
    }
}

impl Debug for PciId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PciId")
            .field("vendor_id", &format!("{:04x}", &self.vendor_id()))
            .field("device_id", &format!("{:04x}", &self.device_id()))
            .finish()
    }
}

impl Serialize for PciId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for PciId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = PciId;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a PCI ID")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                v.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

impl StepLite for PciId {
    fn add_one(&self) -> Self {
        Self {
            value: self.value + 1,
        }
    }

    fn sub_one(&self) -> Self {
        Self {
            value: self.value - 1,
        }
    }
}

impl FromStr for PciId {
    type Err = ParsePciIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (vendor_id, device_id) = s.split_once(':').ok_or(ParsePciIdError::MissingColon)?;
        let vendor_id = u16::from_str_radix(vendor_id, 16)
            .map_err(|parse_int_error| ParsePciIdError::InvalidVendorId(parse_int_error))?;
        let device_id = u16::from_str_radix(device_id, 16)
            .map_err(|parse_int_error| ParsePciIdError::InvalidDeviceId(parse_int_error))?;
        Ok(Self::new(vendor_id, device_id))
    }
}

impl Display for ParsePciIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsePciIdError::InvalidVendorId(parse_int_error) => {
                write!(f, "Invalid Vendor ID. Please refer to {}", parse_int_error)
            }
            ParsePciIdError::InvalidDeviceId(parse_int_error) => {
                write!(f, "Invalid Device ID. Please refer to {}", parse_int_error)
            }
            ParsePciIdError::MissingColon => {
                write!(f, "Invalid PCI ID. Please ensure that the Vendor and Device IDs are separated by a colon `:`")
            }
        }
    }
}

impl Default for ConfigFormat {
    fn default() -> Self {
        return ConfigFormat::Ini;
    }
}

impl Default for ScriptKind {
    fn default() -> Self {
        return Self::Shell;
    }
}

// endregion: IMPLEMENTATIONS