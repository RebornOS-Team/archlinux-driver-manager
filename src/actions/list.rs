use crate::{
    commandline::{CommandlinePrint, ListActionArguments},
    data::{DriverDatabase, HardwareKind},
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
use crate::arch::PackageManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ListActionOutput {
    inner: HashMap<HardwareKind, Vec<InstalledPackage>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    name: String,
    version: String,
}

impl ListActionOutput {
    pub fn new() -> Self {
        ListActionOutput {
            inner: HashMap::<HardwareKind, Vec<InstalledPackage>>::new(),
        }
    }
}

impl Deref for ListActionOutput {
    type Target = HashMap<HardwareKind, Vec<InstalledPackage>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ListActionOutput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for ListActionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl CommandlinePrint for ListActionOutput {
    fn print(&self) {
        for hardware in self.inner.iter() {
            println!(
                "{}",
                hardware.0.if_supports_color(Stdout, |text| text.bold())
            );
            for package in hardware.1.iter() {
                println!(
                    "\t{} {}",
                    package.name.if_supports_color(Stdout, |text| text.yellow()),
                    package
                        .version
                        .if_supports_color(Stdout, |text| text.green())
                );
            }
        }
    }

    fn print_json(&self) {
        println!("{}", serde_json::to_string(&self).unwrap_or_else(|err| {
            eprintln!("The output could not be converted to JSON. Please try another output format...");
            String::from("")
        }));
    }

    fn print_plain(&self) {
        for hardware in self.inner.iter() {
            for package in hardware.1.iter() {
                println!(
                    "{} {} {}",
                    hardware.0.to_string().to_lowercase(),
                    package.name,
                    package
                        .version                        
                );
            }
        }
    }

    fn print_debug(&self) {
        self.print();
    }
}

pub fn list(list_action_arguments: ListActionArguments) -> Result<ListActionOutput, Error> {
    let db = DriverDatabase::try_with_database_path(list_action_arguments.database_file)?;
    db.load().context(DatabaseSnafu {})?;
    let mut list_action_output = ListActionOutput::new();
    match &list_action_arguments.hardware {
        Some(hardware) => db
            .read(|db| {
                if let Some(driver_listing) = db.get(hardware) {
                    list_action_output.entry(*hardware).or_insert(
                        driver_listing
                            .all_packages()
                            .iter()
                            .map(|package| InstalledPackage {
                                name: package.to_owned(),
                                version: String::from("0.0.0"),
                            })
                            .collect(),
                    );
                }
            })
            .context(DatabaseSnafu {})?,
        None => db
            .read(|db| {
                db.iter().for_each(|item| {
                    list_action_output.entry(*item.0).or_insert(
                        item.1
                            .all_packages()
                            .iter()
                            .map(|package| InstalledPackage {
                                name: package.to_owned(),
                                version: String::from("0.0.0"),
                            })
                            .collect(),
                    );
                });
            })
            .context(DatabaseSnafu {})?,
    }

    let package_manager = PackageManager::new();
    println!("{:#?}", package_manager.get("linux"));
    println!("{:#?}", package_manager.get("pacman"));
    println!("{:#?}", package_manager.get("nvidia"));
    println!("{:#?}", package_manager.get("nvidia-dkms"));
    
    Ok(list_action_output)
}   
