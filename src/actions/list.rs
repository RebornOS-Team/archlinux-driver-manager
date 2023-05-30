use crate::arch::PackageManager;
use crate::data::HardwareKind;
use crate::{
    cli::{CommandlinePrint, ListActionArguments},
    data::database::DriverDatabase,
    error::{DatabaseSnafu, Error},
};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Display;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ListActionOutput {
    inner: HashMap<HardwareKind, HashSet<InstalledPackage>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
}

impl ListActionOutput {
    pub fn new() -> Self {
        ListActionOutput {
            inner: HashMap::<HardwareKind, HashSet<InstalledPackage>>::new(),
        }
    }
}

impl Deref for ListActionOutput {
    type Target = HashMap<HardwareKind, HashSet<InstalledPackage>>;

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
        for (hardware_kind, installed_packages) in self.inner.iter() {
            println!(
                "{}",
                hardware_kind.if_supports_color(Stdout, |text| text.bold())
            );
            for package in installed_packages.iter() {
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
        for (hardware_kind, installed_packages) in self.inner.iter() {
            for package in installed_packages.iter() {
                println!(
                    "{} {} {}",
                    hardware_kind.to_string().to_lowercase(),
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

fn all_driver_packages(
    optional_hardware: Option<HardwareKind>,
    filter_tags: &BTreeSet<String>,
    driver_database: &DriverDatabase,
) -> Result<HashMap<HardwareKind, HashSet<String>>, Error> {
    let mut all_driver_packages = HashMap::<HardwareKind, HashSet<String>>::new();
    match optional_hardware {
        Some(hardware) => driver_database
            .read(|hardware_listing| {
                if let Some(driver_listing) = hardware_listing.get(&hardware) {
                    all_driver_packages
                        .entry(hardware.to_owned())
                        .or_default()
                        .extend(driver_listing.all_package_names(filter_tags));
                }
            })
            .context(DatabaseSnafu {})?,
        None => driver_database
            .read(|hardware_listing| {
                all_driver_packages.extend(hardware_listing.all_packages(filter_tags));
            })
            .context(DatabaseSnafu {})?,
    }

    Ok(all_driver_packages)
}

fn installed_drivers(
    all_driver_packages: HashMap<HardwareKind, HashSet<String>>,
    package_manager: &PackageManager,
) -> HashMap<HardwareKind, HashSet<InstalledPackage>> {
    let mut installed_drivers = HashMap::<HardwareKind, HashSet<InstalledPackage>>::new();
    for (hardware_kind, package_names) in all_driver_packages {
        installed_drivers.entry(hardware_kind).or_default().extend(
            package_names.iter().filter_map(|package_name| {
                package_manager
                    .get(package_name)
                    .map(|package| InstalledPackage {
                        name: package.name().to_owned(),
                        version: package.version().to_string(),
                    })
            }),
        );
    }
    installed_drivers
}

pub fn list_inner<T: IntoIterator<Item = String>>(
    database_filepath: PathBuf,
    optional_hardware: Option<HardwareKind>,
    tags: T,
) -> Result<HashMap<HardwareKind, HashSet<InstalledPackage>>, Error> {
    let driver_database = DriverDatabase::with_database_path(database_filepath)?;
    let package_manager = PackageManager::new();

    driver_database.load().context(DatabaseSnafu {})?;
    let all_driver_packages = all_driver_packages(
        optional_hardware,
        &tags.into_iter().collect(),
        &driver_database,
    )?;

    Ok(installed_drivers(all_driver_packages, &package_manager))
}

pub fn list(list_action_arguments: ListActionArguments) -> Result<ListActionOutput, Error> {
    Ok(ListActionOutput {
        inner: list_inner(
            list_action_arguments.database_file,
            list_action_arguments.hardware,
            list_action_arguments.tags,
        )?,
    })
}
