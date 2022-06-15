use crate::{
    commandline::{CommandlinePrint, SearchActionArguments},
    data::database::{DriverDatabase, DriverRecord, HardwareId, HardwareKind, PciId, UsbId},
    error::{DatabaseSnafu, Error},
};
use aparato::{Device, Fetch, PCIDevice};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use std::{
    collections::{BTreeSet, HashSet},
    fmt::Display,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SearchActionOutput {
    inner: HashMap<HardwareKind, HashSet<DriverRecord>>,
}

impl SearchActionOutput {
    pub fn new() -> Self {
        SearchActionOutput {
            inner: HashMap::<HardwareKind, HashSet<DriverRecord>>::new(),
        }
    }
}

impl Deref for SearchActionOutput {
    type Target = HashMap<HardwareKind, HashSet<DriverRecord>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SearchActionOutput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for SearchActionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl CommandlinePrint for SearchActionOutput {
    fn print(&self) {
        for (hardware_kind, driver_records) in self.inner.iter() {
            println!(
                "{}",
                hardware_kind.if_supports_color(Stdout, |text| text.bold())
            );
            println!("");
            for driver_record in driver_records.iter() {
                println!(
                    "\t{}",
                    driver_record
                        .name
                        .if_supports_color(Stdout, |text| text.yellow())
                );
                println!(
                    "\t{} {:?}",
                    "Search tags:".if_supports_color(Stdout, |text| text.green()),
                    driver_record
                        .tags
                );                
                println!(
                    "\t{} {}",
                    "Description:".if_supports_color(Stdout, |text| text.green()),
                    driver_record
                        .description
                );
                println!(
                    "\t{} {:?}",
                    "Packages:".if_supports_color(Stdout, |text| text.green()),
                    driver_record
                        .packages
                );
                println!("");
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
        for (hardware_kind, driver_records) in self.inner.iter() {
            for driver_record in driver_records.iter() {
                println!(
                    "{} {} {:?} {} {:?}",
                    hardware_kind.to_string().to_lowercase(),
                    driver_record.name,
                    driver_record.tags,                    
                    driver_record.description,
                    driver_record.packages,
                );
            }
        }
    }

    fn print_debug(&self) {
        self.print();
    }
}

fn hardware_ids_present() -> BTreeSet<HardwareId> {
    let mut hardware_ids_present = BTreeSet::<HardwareId>::new();

    let pci_ids_present = PCIDevice::fetch(None).into_iter().map(|item| {
        HardwareId::Pci(PciId {
            vendor_id: {
                let vendor_id_byte_array = item.vendor_id();
                (vendor_id_byte_array[0] as u16) * 16u16.pow(2) + (vendor_id_byte_array[1] as u16)
            },
            device_id: {
                let device_id_byte_array = item.device_id();
                (device_id_byte_array[0] as u16) * 16u16.pow(2) + (device_id_byte_array[1] as u16)
            },
        })
    });

    let usb_ids_present = usb_enumeration::enumerate(None, None)
        .into_iter()
        .map(|item| {
            HardwareId::Usb(UsbId {
                vendor_id: item.vendor_id,
                device_id: item.product_id,
            })
        });

    hardware_ids_present.extend(pci_ids_present);
    hardware_ids_present.extend(usb_ids_present);

    hardware_ids_present
}

pub fn search(search_action_arguments: SearchActionArguments) -> Result<SearchActionOutput, Error> {
    let driver_database =
        DriverDatabase::with_database_path(search_action_arguments.database_file)?;
    driver_database.load().context(DatabaseSnafu {})?;

    let hardware_ids_present = hardware_ids_present();

    let mut relevant_driver_records = HashMap::<HardwareKind, HashSet<DriverRecord>>::new();
    if let Some(hardware_kind) = search_action_arguments.hardware {
        driver_database
            .read(|hardware_listing| {
                if let Some(driver_listing) = hardware_listing.get(&hardware_kind) {
                    for (hardware_ids, driver_records) in driver_listing.iter() {
                        if !hardware_ids.is_disjoint(&hardware_ids_present) {
                            relevant_driver_records
                                .entry(hardware_kind)
                                .or_default()
                                .extend(driver_records.clone().into_iter());
                        }
                    }
                }
            })
            .unwrap();
    } else {
        driver_database
            .read(|hardware_listing| {
                for (hardware_kind, driver_listing) in hardware_listing.iter() {
                    for (hardware_ids, driver_records) in driver_listing.iter() {
                        if !hardware_ids.is_disjoint(&hardware_ids_present) {
                            relevant_driver_records
                                .entry(hardware_kind.to_owned())
                                .or_default()
                                .extend(driver_records.clone().into_iter());
                        }
                    }
                }
            })
            .unwrap();
    }

    Ok(SearchActionOutput {
        inner: relevant_driver_records,
    })
}
