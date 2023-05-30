use crate::error::{DatabaseSnafu, Error};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Clone)]
pub struct DriverDatabase {
    pub db: jammdb::DB,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HardwareId {
    #[serde(alias = "PCI", alias = "pci")]
    Pci(PciId),

    #[serde(alias = "USB", alias = "usb")]
    Usb(UsbId),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PciId {
    #[serde(alias = "vendor-id")]
    pub vendor: u16,

    #[serde(alias = "device-id")]
    pub device: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UsbId {
    #[serde(alias = "vendor-id")]
    pub vendor: u16,

    #[serde(alias = "device-id")]
    pub device: u16,
}

impl DriverDatabase {
    pub fn with_database_path(filepath: PathBuf) -> Result<Self, Error> {
        let mut temp_database_file = tempfile::NamedTempFile::new().expect(
            "Could not create a temporary file with write permissions to create a database.",
        );
        std::io::copy(
            &mut std::fs::File::open(filepath).expect("Could not open the database file."),
            &mut temp_database_file,
        )
        .unwrap();
        Ok(DriverDatabase {
            db: jammdb::DB::open(temp_database_file.into_temp_path()).context(DatabaseSnafu {})?,
        })
    }
}

impl Deref for DriverDatabase {
    type Target = jammdb::DB;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for DriverDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

pub fn convert_tag<S: AsRef<str>>(tag: S) -> String {
    tag.as_ref().trim().replace("-", " ").replace("_", " ")
}
