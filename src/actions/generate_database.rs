use crate::{
    cli::{CommandlinePrint, GenerateDatabaseActionArguments},
    data::{database, input_file},
    error::{DatabaseSnafu, Error},
};
use owo_colors::{OwoColorize, Stream::Stdout};
use rmp_serde;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::path::PathBuf;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GenerateDatabaseActionOutput {
    success_message: String,
}

impl GenerateDatabaseActionOutput {
    pub fn new() -> Self {
        GenerateDatabaseActionOutput {
            success_message: "Database generated successfully...".to_string(),
        }
    }
}

impl CommandlinePrint for GenerateDatabaseActionOutput {
    fn print(&self) {
        println!(
            "{}",
            self.success_message
                .if_supports_color(Stdout, |text| text.green())
        );
    }

    fn print_json(&self) {
        println!("{}", serde_json::to_string(&self).unwrap_or_else(|_| {
            eprintln!("The output could not be converted to JSON. Please try another output format...");
            String::from("")
        }));
    }

    fn print_plain(&self) {
        println!("{}", self.success_message);
    }

    fn print_debug(&self) {
        self.print();
    }
}

pub fn generate_database_inner(
    input_file: PathBuf,
    database_file: PathBuf,
) -> Result<GenerateDatabaseActionOutput, Error> {
    let input_driver_listing = input_file::parse_input_file(input_file)?;
    let driver_database = database::DriverDatabase::with_database_path(database_file)?;

    // open a writable transaction so we can make changes
    let transaction = driver_database.tx(true).context(DatabaseSnafu)?;

    let pci_ids_to_hardware_case_ids_bucket = transaction
        .create_bucket("pci_ids_to_hardware_case_ids_bucket")
        .context(DatabaseSnafu)?;

    let usb_ids_to_hardware_case_ids_bucket = transaction
        .create_bucket("usb_ids_to_hardware_case_ids_bucket")
        .context(DatabaseSnafu)?;

    let hardware_case_ids_to_driver_options_bucket: jammdb::Bucket = transaction
        .create_bucket("hardware_case_ids_to_driver_options")
        .context(DatabaseSnafu)?;

    for hardware_case in input_driver_listing {
        hardware_case_ids_to_driver_options_bucket
            .put(
                hardware_case.id,
                rmp_serde::to_vec(&hardware_case.driver_options).unwrap(),
            )
            .context(DatabaseSnafu)?;
        for hardware_group in hardware_case.hardware_groups {
            for device_entry in hardware_group.device_entries {
                match device_entry {
                    input_file::DeviceEntry::Pci(pci_id_list) => {
                        for device in pci_id_list.devices {
                            let hardware_id: u32 =
                                ((pci_id_list.vendor as u32) << 16) | device as u32;
                            pci_ids_to_hardware_case_ids_bucket
                                .put(hardware_id.to_be_bytes(), hardware_case.id)
                                .context(DatabaseSnafu)?;
                        }
                    }
                    input_file::DeviceEntry::Usb(usb_id_list) => {
                        for device in usb_id_list.devices {
                            let hardware_id: u32 =
                                ((usb_id_list.vendor as u32) << 16) | device as u32;
                            usb_ids_to_hardware_case_ids_bucket
                                .put(hardware_id.to_be_bytes(), hardware_case.id)
                                .context(DatabaseSnafu)?;
                        }
                    }
                }
            }
        }
    }

    Ok(GenerateDatabaseActionOutput::new())
}

pub fn generate_database(
    generate_database_action_arguments: GenerateDatabaseActionArguments,
) -> Result<GenerateDatabaseActionOutput, Error> {
    Ok(generate_database_inner(
        generate_database_action_arguments.input_file,
        generate_database_action_arguments.database_file,
    )?)
}
