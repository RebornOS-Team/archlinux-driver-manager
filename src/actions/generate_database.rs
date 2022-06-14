use std::collections::BTreeSet;

use crate::data::{database, input_file};
use crate::{
    commandline::{CommandlinePrint, GenerateDatabaseActionArguments},
    error::Error,
};
use owo_colors::{OwoColorize, Stream::Stdout};
use rustbreak::FileDatabase;
use serde::{Deserialize, Serialize};

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

fn convert_hardware_ids(
    input_hardware_ids: Vec<input_file::HardwareIdEntry>,
) -> BTreeSet<database::HardwareId> {
    let btree_set = BTreeSet::<database::HardwareId>::new();
    for input_hardware_id in input_hardware_ids {
        match input_hardware_id {
            input_file::HardwareIdEntry::Pci(pci_id_list) => {
                let vendor_id = pci_id_list.vendor.parse().unwrap_or_else(|vendor| {
                    println!(
                        "{}: {}",
                        "Error: Invalid vendor ID".if_supports_color(Stdout, |text| text.red()),
                        vendor
                    );
                    0x0000
                });
                for device in pci_id_list.devices {
                    btree_set.insert(database::HardwareId::Pci(database::PciId {
                        vendor_id: vendor_id,
                        device_id: device.parse().unwrap_or_else(|device| {
                            println!(
                                "{}: {}",
                                "Error: Invalid PCI device ID"
                                    .if_supports_color(Stdout, |text| text.red()),
                                device
                            );
                            0x0000
                        }),
                    }));
                }
            }
            input_file::HardwareIdEntry::Usb(usb_id_list) => {
                let vendor_id = usb_id_list.vendor.parse().unwrap_or_else(|vendor| {
                    println!(
                        "{}: {}",
                        "Error: Invalid vendor ID".if_supports_color(Stdout, |text| text.red()),
                        vendor
                    );
                    0x0000
                });
                for device in usb_id_list.devices {
                    btree_set.insert(database::HardwareId::Usb(database::UsbId {
                        vendor_id: vendor_id,
                        device_id: device.parse().unwrap_or_else(|device| {
                            println!(
                                "{}: {}",
                                "Error: Invalid USB device ID"
                                    .if_supports_color(Stdout, |text| text.red()),
                                device
                            );
                            0x0000
                        }),
                    }));
                }
            }
        }
    }
    btree_set
}

pub fn generate_database(
    generate_database_action_arguments: GenerateDatabaseActionArguments,
) -> Result<GenerateDatabaseActionOutput, Error> {
    let input_driver_listing =
        input_file::parse_input_file(generate_database_action_arguments.input_file)?;
    let driver_database = database::DriverDatabase::create_with_database_path(
        generate_database_action_arguments.database_file,
    )?;
    driver_database
        .write(|hardware_listing| {
            for driver_entry in input_driver_listing {
                let driver_listing = hardware_listing
                    .entry(driver_entry.hardware_kind.into())
                    .or_default();

                let hardware_id_set = convert_hardware_ids(driver_entry.ids);

                let driver_records = driver_listing.entry(hardware_id_set).or_default();

                driver_records.push(database::DriverRecord {
                    name: driver_entry.name,
                    description: driver_entry.description,
                    tags: driver_entry.tags,
                    packages: driver_entry.packages,
                    configurations: driver_entry.configurations,
                    pre_install_script: (),
                    post_install_script: (),
                });
            }
        })
        .unwrap();
    Ok(GenerateDatabaseActionOutput::new())
}
