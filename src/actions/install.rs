use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::commandline::{CommandlinePrint, InstallActionArguments};
use crate::error::Error;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct InstallActionOutput {
    inner: Vec<(String, String)>,
}

impl Display for InstallActionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl CommandlinePrint for InstallActionOutput {
    fn print(&self) {}

    fn print_json(&self) {}

    fn print_plain(&self) {}

    fn print_debug(&self) {}
}

pub fn install(install_arguments: InstallActionArguments) -> Result<InstallActionOutput, Error> {
    Ok(InstallActionOutput::default())
}
