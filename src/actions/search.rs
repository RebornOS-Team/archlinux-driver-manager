use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::commandline::{SearchActionArguments, CommandlinePrint};
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
    // let db = DriverDatabase::try_new().unwrap();
    // println!("Writing to Database");
    // db.write(|db| {
    //     db.insert(
    //         PciId::range_inclusive("1234:5678", "1234:56ab")
    //             .expect("Invalid PCI IDs supplied"),
    //         vec![DriverRecord::default()],
    //     );
    //     println!("Entries: \n{:#?}", db);
    // })
    // .unwrap();

    // println!("Syncing Database");
    // db.save().unwrap();

    // println!("Loading Database");
    // db.load().unwrap();

    // println!("Reading from Database");
    // db.read(|db| {
    //     println!("Results:");
    //     println!("{:#?}", db);
    // })
    // .unwrap();
    Ok(
        SearchActionOutput::default()
    )
}