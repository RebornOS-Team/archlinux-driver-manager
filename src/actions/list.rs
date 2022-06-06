use std::fmt::Display;

use serde::{Serialize, Deserialize};
use snafu::ResultExt;

use crate::commandline::{ListActionArguments, CommandlinePrint};
use crate::data::{DriverDatabase, DriverRecord};
use crate::error::{Error, DatabaseSnafu};

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

pub fn list(list_action_arguments: ListActionArguments) -> Result<ListActionOutput, Error> {
    let db = DriverDatabase::try_with_database_path(list_action_arguments.database_file)?;
    db.load().context(DatabaseSnafu{})?;
    match &list_action_arguments.hardware {
        Some(hardware) => {
            db.read(|db| {
                // if let Some(driver_listing) = db.get(hardware) {
                //     driver_listing.iter().fold(Vec::<String>::new(), |acc, x| {
                //         // acc.append()
                //     } )
                // }
            });
        },
        None => todo!(),
    }
    // TODO: Make ListActionOutput smarter with a HashMap pointing to a list instead
    Ok(
        ListActionOutput::default()
    )
}