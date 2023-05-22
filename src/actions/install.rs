use crate::{
    actions::list::list_inner,
    actions::search::search_inner,
    arch::PackageManager,
    commandline::{CommandlinePrint, InstallActionArguments},
    data::database::{DriverRecord, HardwareKind},
    error::Error,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct InstallActionOutput {}

impl CommandlinePrint for InstallActionOutput {
    fn print(&self) {}

    fn print_json(&self) {}

    fn print_plain(&self) {}

    fn print_debug(&self) {}
}

pub fn install_inner<T: IntoIterator<Item = String>>(
    database_filepath: PathBuf,
    hardware: HardwareKind,
    tags: T,
    _enable_aur: bool,
) -> Result<InstallActionOutput, Error> {
    let relevant_driver_records = search_inner(database_filepath.clone(), Some(hardware), tags)?
        .into_values()
        .collect::<Vec<BTreeSet<DriverRecord>>>()
        .pop()
        .unwrap();

    let packages_to_install = relevant_driver_records
        .iter()
        .next()
        .expect("Error: Nothing to install")
        .packages
        .clone();
    let packages_to_remove = list_inner(database_filepath.clone(), Some(hardware), None).map_or(
        Vec::<String>::new(),
        |installed_hash_map| {
            installed_hash_map.into_iter().fold(
                Vec::<String>::new(),
                |mut acc, (_hardware_kind, hash_set)| {
                    acc.append(hash_set.into_iter().fold(
                        &mut Vec::<String>::new(),
                        |acc, installed_package| {
                            if !packages_to_install.contains(&installed_package.name) {
                                acc.push(installed_package.name);
                            }
                            acc
                        },
                    ));
                    acc
                },
            )
        },
    );
    let mut package_manager = PackageManager::new();
    package_manager.install(packages_to_install, packages_to_remove)?;

    Ok(InstallActionOutput::default())
}

pub fn install(
    install_action_arguments: InstallActionArguments,
) -> Result<InstallActionOutput, Error> {
    sudo::escalate_if_needed().expect("ERROR: Could not get superuser privileges...");
    Ok(install_inner(
        install_action_arguments.database_file,
        install_action_arguments.hardware,
        install_action_arguments.tags,
        install_action_arguments.enable_aur,
    )?)
}
