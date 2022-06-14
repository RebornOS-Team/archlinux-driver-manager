use crate::{
    commandline::{CommandlinePrint, SearchActionArguments},
    data::database::{DriverRecord, DriverDatabase, HardwareKind},
    error::{DatabaseSnafu, Error},
};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::fmt::Display;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use aparato::{PCIDevice, Fetch};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SearchActionOutput {
    inner: HashMap<HardwareKind, Vec<DriverRecord>>,
}

impl SearchActionOutput {
    pub fn new() -> Self {
        SearchActionOutput {
            inner: HashMap::<HardwareKind, Vec<DriverRecord>>::new(),
        }
    }
}

impl Deref for SearchActionOutput {
    type Target = HashMap<HardwareKind, Vec<DriverRecord>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SearchActionOutput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for SearchActionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl CommandlinePrint for SearchActionOutput {
    fn print(&self) {
        for hardware in self.inner.iter() {
            println!(
                "{}",
                hardware.0.if_supports_color(Stdout, |text| text.bold())
            );
            println!("");
            for driver_record in hardware.1.iter() {
                println!(
                    "\tSearch tags: {:?}",
                    driver_record.tags.if_supports_color(Stdout, |text| text.yellow())
                );
                println!(
                    "\tName: {}",
                    driver_record.name.if_supports_color(Stdout, |text| text.yellow())
                );
                println!(
                    "\tDescription: {}",
                    driver_record.description.if_supports_color(Stdout, |text| text.yellow())
                );
                println!(
                    "\tPackages: {:?}",
                    driver_record.packages.if_supports_color(Stdout, |text| text.yellow())
                );                
                println!("");
            }
        }
    }

    fn print_json(&self) {
        println!("{}", serde_json::to_string(&self).unwrap_or_else(|_| {
            eprintln!("The output could not be converted to JSON. Please try another output format...");
            String::from("")
        }));
    }

    fn print_plain(&self) {
        for hardware in self.inner.iter() {
            for driver_record in hardware.1.iter() {
                println!(
                    "{} {:?} {} {} {:?}",
                    hardware.0.to_string().to_lowercase(),
                    driver_record.tags,
                    driver_record.name,
                    driver_record.description,
                    driver_record.packages,
                );
            }
        }
    }

    fn print_debug(&self) {
        self.print();
    }
}

pub fn search(search_action_arguments: SearchActionArguments) -> Result<SearchActionOutput, Error> {
    let driver_database =
        DriverDatabase::load_with_database_path(search_action_arguments.database_file)?;

    driver_database.load().context(DatabaseSnafu {})?;
    
    println!("{:#?}", PCIDevice::fetch(Some(25)));

    Ok(SearchActionOutput {
        inner: HashMap::<HardwareKind, Vec<DriverRecord>>::new(),
    })
}

