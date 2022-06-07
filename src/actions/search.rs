use std::fmt::Display;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::commandline::{SearchActionArguments, CommandlinePrint};
use crate::data::{DriverDatabase, DriverRecord, PciId, HardwareKind, DriverListing};
use crate::error::Error;

#[derive(
    Default,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct SearchActionOutput {
    inner: Vec<(String, String)>
}

impl Display for SearchActionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl CommandlinePrint for SearchActionOutput {
    fn print(&self) {
    }

    fn print_json(&self) {
    }

    fn print_plain(&self) {
    }

    fn print_debug(&self) {
    }
}

pub fn search(search_arguments: SearchActionArguments) -> Result<SearchActionOutput, Error>{
    let db = DriverDatabase::try_with_database_path(PathBuf::from("driver_database.ron")).unwrap();
    println!("Writing to Database");
    db.write(|db| {
        db.insert(
            HardwareKind::Graphics,
            {
                let mut driver_listing = DriverListing::new();
                driver_listing.insert(PciId::range_inclusive("abc1:fab2", "afa2:aaba").unwrap(), vec![
                    DriverRecord::default(),
                ]);
                driver_listing
            },
        );
        db.insert(
            HardwareKind::Wireless,
            {
                let mut driver_listing = DriverListing::new();
                driver_listing.insert(PciId::range_inclusive("aaba:fab2", "abaa:1231").unwrap(), vec![
                    DriverRecord::default(),
                ]);
                driver_listing.insert(PciId::range_inclusive("abaa:1241", "abaa:1251").unwrap(), vec![
                    DriverRecord::default(),
                ]);
                driver_listing
            },
        );
        println!("Entries: \n{:#?}", db);
    })
    .unwrap();

    println!("Syncing Database");
    db.save().unwrap();

    println!("Loading Database");
    db.load().unwrap();

    println!("Reading from Database");
    db.read(|db| {
        println!("Results:");
        println!("{:#?}", db);
    })
    .unwrap();
    Ok(
        SearchActionOutput::default()
    )
}