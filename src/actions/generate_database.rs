use crate::{
    cli::{CommandlinePrint, GenerateDatabaseActionArguments},
    data::{database, input_file},
    error::Error,
};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

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
    let mut btree_set = BTreeSet::<database::HardwareId>::new();
    for input_hardware_id in input_hardware_ids {
        match input_hardware_id {
            input_file::HardwareIdEntry::Pci(pci_id_list) => {
                let vendor = pci_id_list.vendor;
                for device in pci_id_list.devices {
                    btree_set.insert(database::HardwareId::Pci(
                        (vendor.as_ref(), device.as_ref())
                            .try_into()
                            .unwrap_or_else(|error| {
                                println!(
                                    "{}({}:{})...{}",
                                    "Error: Invalid PCI ID "
                                        .if_supports_color(Stdout, |text| text.red()),
                                    vendor.if_supports_color(Stdout, |text| text.bold()),
                                    device.if_supports_color(Stdout, |text| text.bold()),
                                    error
                                );
                                database::PciId {
                                    vendor_id: 0x0000,
                                    device_id: 0x0000,
                                }
                            }),
                    ));
                }
            }
            input_file::HardwareIdEntry::Usb(usb_id_list) => {
                let vendor = usb_id_list.vendor;
                for device in usb_id_list.devices {
                    btree_set.insert(database::HardwareId::Usb(
                        (vendor.as_ref(), device.as_ref())
                            .try_into()
                            .unwrap_or_else(|error| {
                                println!(
                                    "{}({}:{})...{}",
                                    "Error: Invalid USB ID "
                                        .if_supports_color(Stdout, |text| text.red()),
                                    vendor.if_supports_color(Stdout, |text| text.bold()),
                                    device.if_supports_color(Stdout, |text| text.bold()),
                                    error
                                );
                                database::UsbId {
                                    vendor_id: 0x0000,
                                    device_id: 0x0000,
                                }
                            }),
                    ));
                }
            }
        }
    }
    btree_set
}

pub fn generate_database_inner(
    input_file: PathBuf,
    database_file: PathBuf,
) -> Result<GenerateDatabaseActionOutput, Error> {
    let input_driver_listing = input_file::parse_input_file(input_file)?;
    let driver_database = database::DriverDatabase::create_with_database_path(database_file)?;
    driver_database
        .write(|hardware_listing| {
            for driver_entry in input_driver_listing {
                let driver_listing = hardware_listing
                    .entry(driver_entry.hardware_kind.into())
                    .or_default();

                let hardware_id_set = convert_hardware_ids(driver_entry.ids);

                let driver_records = driver_listing.entry(hardware_id_set).or_default();

                driver_records.insert(database::DriverRecord {
                    order_of_priority: driver_entry.order_of_priority,
                    name: driver_entry.name,
                    description: driver_entry.description,
                    tags: driver_entry
                        .tags
                        .iter()
                        .map(database::convert_tag)
                        .collect(),
                    packages: driver_entry.packages,
                    configurations: driver_entry
                        .configurations
                        .into_iter()
                        .map(|item| item.into())
                        .collect(),
                    pre_install_script: driver_entry.pre_install.map(|item| item.into()),
                    post_install_script: driver_entry.post_install.map(|item| item.into()),
                });
            }
        })
        .unwrap();
    driver_database.save().unwrap();

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
