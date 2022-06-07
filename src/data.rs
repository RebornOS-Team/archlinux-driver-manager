use crate::error::{DatabaseSnafu, Error};
use rangemap::{RangeInclusiveMap, StepLite};
use rustbreak::{deser::Ron, FileDatabase};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use snafu::ResultExt;
use std::num::ParseIntError;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut, Range, RangeInclusive},
    path::PathBuf,
    str::FromStr,
};

// region: DATA MODEL
#[derive(Debug)]
pub struct DriverDatabase {
    inner: FileDatabase<HardwareListing, Ron>,
}

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct HardwareListing {
    inner: HashMap<HardwareKind, DriverListing>,
}

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct DriverListing {
    inner: RangeInclusiveMap<PciId, Vec<DriverRecord>>,
}

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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, clap::ArgEnum)]
pub enum HardwareKind {
    Graphics,
    Ethernet,
    Wireless,
    Sound,
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
    Clone,    
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

impl HardwareKind {
    pub fn all_to_strings() -> Vec<String> {
        vec![
            String::from("graphics"),
            String::from("ethernet"),
            String::from("wireless"),
            String::from("sound"),
        ]
    }
}

impl Default for HardwareKind {
    fn default() -> Self {
        HardwareKind::Graphics
    }
}

impl Display for HardwareKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &HardwareKind::Graphics => write!(f, "Graphics"),
            &HardwareKind::Ethernet => write!(f, "Ethernet"),
            &HardwareKind::Wireless => write!(f, "Wireless"),
            &HardwareKind::Sound => write!(f, "Sound"),
        }
    }
}

impl DriverDatabase {
    pub fn try_with_database_path(filepath: PathBuf) -> Result<Self, Error> {
        Ok(DriverDatabase {
            inner: FileDatabase::<HardwareListing, Ron>::load_from_path_or_default(filepath)
                .context(DatabaseSnafu {})?,
        })
    }
}

impl Deref for DriverDatabase {
    type Target = FileDatabase<HardwareListing, Ron>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DriverDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl HardwareListing {
    pub fn new() -> Self {
        Self {
            inner: HashMap::<HardwareKind, DriverListing>::new(),
        }
    }

    pub fn all_packages(&self) -> Vec<String> {
        let mut packages = Vec::<String>::new();
        packages.append(self.iter().fold(
            &mut Vec::<String>::new(),
            |acc, x| {
                acc.append(&mut x.1.all_packages());
                acc
            },
        ));
        packages
    }
}

impl Deref for HardwareListing {
    type Target = HashMap<HardwareKind, DriverListing>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for HardwareListing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Default for HardwareListing {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl DriverListing {
    pub fn new() -> Self {
        Self {
            inner: RangeInclusiveMap::<PciId, Vec<DriverRecord>>::new(),
        }
    }

    pub fn all_packages(&self) -> Vec<String> {
        let mut packages = Vec::<String>::new();
        packages.append(self.iter().fold(
            &mut Vec::<String>::new(),
            |acc, x| {
                acc.append(x.1.iter().fold(&mut Vec::<String>::new(), |acc, x| {
                    acc.append(&mut x.packages.clone());
                    acc
                }));
                acc
            },
        ));
        packages
    }
}

impl Deref for DriverListing {
    type Target = RangeInclusiveMap<PciId, Vec<DriverRecord>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DriverListing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Default for DriverListing {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

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
