use std::collections::BTreeSet;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::{
    commandline::{CommandlinePrint, InstallActionArguments},
    data::database::{DriverRecord, HardwareKind},
    error::Error,
    actions::search::search_inner,
    arch::PackageManager,
};

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
    enable_aur: bool,
) -> Result<InstallActionOutput, Error> {
    let relevant_driver_records = search_inner(database_filepath, Some(hardware), tags)?
        .into_values()
        .collect::<Vec<BTreeSet<DriverRecord>>>().pop().unwrap();
    
    let package_names = relevant_driver_records.iter().next().expect("Error: Nothing to install").packages.clone();
    let mut package_manager = PackageManager::new();
    package_manager.install(package_names)?;

    Ok(InstallActionOutput::default())
}

pub fn install(
    install_action_arguments: InstallActionArguments,
) -> Result<InstallActionOutput, Error> {
    Ok(install_inner(
        install_action_arguments.database_file,
        install_action_arguments.hardware,
        install_action_arguments.tags,
        install_action_arguments.enable_aur,
    )?)
}
