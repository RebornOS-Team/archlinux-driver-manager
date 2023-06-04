use crate::{
    error::{DatabaseSnafu, Error},
    DB_PATH_TEMP,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    fs,
    ops::{Deref, DerefMut},
    path::{self, PathBuf},
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
        Ok(DriverDatabase {
            db: { jammdb::DB::open(filepath).context(DatabaseSnafu)? },
        })
    }

    pub fn cloned_from_database_path(filepath: PathBuf) -> Result<Self, Error> {
        let temp_db_path = path::PathBuf::from(*DB_PATH_TEMP);
        std::fs::create_dir_all(temp_db_path.parent().unwrap()).unwrap();
        _ = std::fs::remove_file(&temp_db_path).ok();
        if filepath.exists() {
            std::io::copy(
                &mut std::fs::File::open(&filepath).expect("Could not open the database file."),
                &mut fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&temp_db_path)
                    .unwrap(),
            )
            .unwrap();
        }
        println!("filepath: {:?}", filepath);
        println!("temp_db_path: {:?}", temp_db_path);
        println!("{:?}", temp_db_path.exists());
        DriverDatabase::with_database_path(temp_db_path)
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
