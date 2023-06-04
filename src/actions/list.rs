use crate::arch::PackageManager;
use crate::data::input_file::{DriverOption, HardwareKind};
use crate::{
    cli::{CommandlinePrint, ListActionArguments},
    data::database::DriverDatabase,
    error::{DatabaseSnafu, Error},
};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::path::PathBuf;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ListActionOutput {
    inner: BTreeMap<HardwareKind, BTreeSet<InstalledPackage>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
}

impl ListActionOutput {
    pub fn new() -> Self {
        ListActionOutput {
            inner: BTreeMap::<HardwareKind, BTreeSet<InstalledPackage>>::new(),
        }
    }
}

impl Deref for ListActionOutput {
    type Target = BTreeMap<HardwareKind, BTreeSet<InstalledPackage>>;

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
    optional_hardware: &Option<HardwareKind>,
    filter_tags: &BTreeSet<String>,
    driver_database: &DriverDatabase,
) -> Result<BTreeMap<HardwareKind, BTreeSet<String>>, Error> {
    // Open a read-only transaction to get the data
    let transaction = driver_database.tx(false).context(DatabaseSnafu {})?;

    let hardware_kind_to_driver_option_id_bucket = transaction
        .get_bucket("hardware_kind_to_driver_option_id_bucket")
        .context(DatabaseSnafu)?;

    let driver_option_id_to_driver_option_bucket = transaction
        .get_bucket("driver_option_id_to_driver_option_bucket")
        .context(DatabaseSnafu)?;

    let process_hardware_kind = |hardware_kinds: &BTreeSet<HardwareKind>| {
        hardware_kinds.into_iter().fold(
            BTreeMap::<HardwareKind, BTreeSet<String>>::new(),
            |grouped_packages: BTreeMap<HardwareKind, BTreeSet<String>>,
             hardware_kind: &HardwareKind| {
                if let Some(data) =
                    hardware_kind_to_driver_option_id_bucket.get(hardware_kind.to_string())
                {
                    let driver_option_ids: BTreeSet<String> =
                        rmp_serde::from_slice(data.kv().value()).unwrap();
                    driver_option_ids
                        .iter()
                        .filter_map(|driver_option_id| {
                            if let Some(driver_option_data) =
                                driver_option_id_to_driver_option_bucket.get(driver_option_id)
                            {
                                rmp_serde::from_slice(driver_option_data.kv().value()).ok()
                            } else {
                                None
                            }
                        })
                        .fold(
                            grouped_packages,
                            |mut inner_grouped_packages, driver_option: DriverOption| {
                                if filter_tags
                                    .into_iter()
                                    .all(|tag| driver_option.tags.contains(tag))
                                {
                                    inner_grouped_packages
                                        .entry(hardware_kind.clone())
                                        .or_default()
                                        .extend(driver_option.packages.into_iter());
                                }
                                inner_grouped_packages
                            },
                        )
                } else {
                    BTreeMap::<HardwareKind, BTreeSet<String>>::new()
                }
            },
        )
    };

    if let Some(hardware_kind) = optional_hardware {
        return Ok(process_hardware_kind(&BTreeSet::from([
            hardware_kind.clone()
        ])));
    } else {
        return Ok(process_hardware_kind(
            &hardware_kind_to_driver_option_id_bucket
                .kv_pairs()
                .filter_map(|data| rmp_serde::from_slice(data.value()).ok())
                .collect::<BTreeSet<HardwareKind>>(),
        ));
    }
}

fn installed_drivers(
    all_driver_packages: &BTreeMap<HardwareKind, BTreeSet<String>>,
    package_manager: &PackageManager,
) -> BTreeMap<HardwareKind, BTreeSet<InstalledPackage>> {
    let mut installed_drivers = BTreeMap::<HardwareKind, BTreeSet<InstalledPackage>>::new();
    for (hardware_kind, package_names) in all_driver_packages {
        installed_drivers
            .entry(hardware_kind.clone())
            .or_default()
            .extend(package_names.iter().filter_map(|package_name| {
                package_manager
                    .get(package_name)
                    .map(|package| InstalledPackage {
                        name: package.name().to_owned(),
                        version: package.version().to_string(),
                    })
            }));
    }
    installed_drivers
}

pub fn list_inner<T: IntoIterator<Item = String>>(
    database_filepath: PathBuf,
    optional_hardware: &Option<HardwareKind>,
    tags: T,
) -> Result<BTreeMap<HardwareKind, BTreeSet<InstalledPackage>>, Error> {
    let driver_database = DriverDatabase::cloned_from_database_path(database_filepath)?;
    let package_manager = PackageManager::new();

    let all_driver_packages = all_driver_packages(
        optional_hardware,
        &tags.into_iter().collect(),
        &driver_database,
    )?;

    Ok(installed_drivers(&all_driver_packages, &package_manager))
}

pub fn list(list_action_arguments: ListActionArguments) -> Result<ListActionOutput, Error> {
    Ok(ListActionOutput {
        inner: list_inner(
            list_action_arguments.database_file,
            &list_action_arguments.hardware,
            list_action_arguments.tags,
        )?,
    })
}
