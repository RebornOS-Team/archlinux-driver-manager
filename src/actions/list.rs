use crate::arch::PackageManager;
use crate::{
    commandline::{CommandlinePrint, ListActionArguments},
    data::database::{DriverDatabase, HardwareKind},
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
        println!("{}", serde_json::to_string(&self).unwrap_or_else(|_| {
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
                    package.version
                );
            }
        }
    }

    fn print_debug(&self) {
        self.print();
    }
}

pub fn list(list_action_arguments: ListActionArguments) -> Result<ListActionOutput, Error> {
    let driver_database =
        DriverDatabase::try_with_database_path(list_action_arguments.database_file)?;
    let package_manager = PackageManager::new();

    driver_database.load().context(DatabaseSnafu {})?;
    let all_driver_packages =
        all_driver_packages(list_action_arguments.hardware, &driver_database)?;

    Ok(ListActionOutput {
        inner: installed_drivers(all_driver_packages, &package_manager),
    })
}

fn all_driver_packages(
    optional_hardware: Option<HardwareKind>,
    driver_database: &DriverDatabase,
) -> Result<HashMap<HardwareKind, Vec<String>>, Error> {
    let mut all_driver_packages = HashMap::<HardwareKind, Vec<String>>::new();
    match optional_hardware {
        Some(hardware) => driver_database
            .read(|hardware_listing| {
                if let Some(driver_listing) = hardware_listing.get(&hardware) {
                    all_driver_packages.insert(hardware.to_owned(), driver_listing.all_packages());
                }
            })
            .context(DatabaseSnafu {})?,
        None => driver_database
            .read(|hardware_listing| {
                all_driver_packages.extend(hardware_listing.all_packages().into_iter());
            })
            .context(DatabaseSnafu {})?,
    }

    Ok(all_driver_packages)
}

fn installed_drivers(
    all_driver_packages: HashMap<HardwareKind, Vec<String>>,
    package_manager: &PackageManager,
) -> HashMap<HardwareKind, Vec<InstalledPackage>> {
    all_driver_packages
        .iter()
        .filter_map(|hardware_entry| {
            let installed_package_list: Vec<InstalledPackage> = hardware_entry
                .1
                .iter()
                .filter_map(|package_name| {
                    package_manager
                        .get(package_name)
                        .map(|package| InstalledPackage {
                            name: package.name().to_owned(),
                            version: package.version().to_string(),
                        })
                })
                .collect();
            if installed_package_list.len() > 0 {
                Some((hardware_entry.0.to_owned(), installed_package_list))
            } else {
                None
            }
        })
        .collect()
}
