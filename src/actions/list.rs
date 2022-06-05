use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::commandline::{ListActionArguments, CommandlinePrint};
use crate::error::Error;

#[derive(
    Default,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
pub struct ListActionOutput {
    inner: Vec<(String, String)>
}

impl Display for ListActionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl CommandlinePrint for ListActionOutput {
    fn print(&self) {
    }

    fn print_json(&self) {
    }

    fn print_plain(&self) {
    }

    fn print_debug(&self) {
    }
}

pub fn list(list_arguments: ListActionArguments) -> Result<ListActionOutput, Error>{
    Ok(
        ListActionOutput::default()
    )
}