use crate::{
    cli::{CommandlinePrint, SearchActionArguments},
    data::database::DriverDatabase,
    data::{
        database::{HardwareId, PciId, UsbId},
        input_file::{DriverOption, HardwareSetup, HardwareKind},
    },
    error::{DatabaseSnafu, Error},
};
use devices;
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{collections::BTreeSet, fmt::Display};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SearchActionOutput {
    inner: HashMap<HardwareKind, BTreeSet<DriverOption>>,
}

impl SearchActionOutput {
    pub fn new() -> Self {
        SearchActionOutput {
            inner: HashMap::<HardwareKind, BTreeSet<DriverOption>>::new(),
        }
    }
}

impl Deref for SearchActionOutput {
    type Target = HashMap<HardwareKind, BTreeSet<DriverOption>>;

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
                    driver_record.tags
                );
                println!(
                    "\t{} {}",
                    "Description:".if_supports_color(Stdout, |text| text.green()),
                    driver_record.description
                );
                println!(
                    "\t{} {:?}",
                    "Packages:".if_supports_color(Stdout, |text| text.green()),
                    driver_record.packages
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

    let pci_ids_present = devices::Devices::get()
        .expect("Failed to get connected devices")
        .into_iter()
        .filter_map(|item| match item.path() {
            devices::DevicePath::PCI {
                bus: _,
                slot: _,
                function: _,
            } => Some(HardwareId::Pci(PciId {
                vendor: item.vendor_id(),
                device: item.product_id(),
            })),
            devices::DevicePath::USB { bus: _, device: _ } => None,
        });

    let usb_ids_present = usb_enumeration::enumerate(None, None)
        .into_iter()
        .map(|item| {
            HardwareId::Usb(UsbId {
                vendor: item.vendor_id,
                device: item.product_id,
            })
        });

    hardware_ids_present.extend(pci_ids_present);
    hardware_ids_present.extend(usb_ids_present);

    hardware_ids_present
}

pub fn search_inner<T: IntoIterator<Item = String>>(
    database_filepath: PathBuf,
    optional_hardware: Option<HardwareKind>,
    tags: T,
) -> Result<HashMap<HardwareKind, BTreeSet<DriverOption>>, Error> {
    let driver_database = DriverDatabase::with_database_path(database_filepath)?;

    // Open a read-only transaction to get the data
    let transaction = driver_database.tx(false).context(DatabaseSnafu {})?;

    let hardware_ids_present = hardware_ids_present();

    let filter_tags: BTreeSet<String> = tags.into_iter().collect();

    let pci_ids_to_hardware_case_ids_bucket = transaction
        .get_bucket("pci_ids_to_hardware_case_ids_bucket")
        .context(DatabaseSnafu)?;

    let usb_ids_to_hardware_case_ids_bucket = transaction
        .get_bucket("usb_ids_to_hardware_case_ids_bucket")
        .context(DatabaseSnafu)?;

    let hardware_case_ids_to_driver_options_bucket: jammdb::Bucket = transaction
        .get_bucket("hardware_case_ids_to_driver_options")
        .context(DatabaseSnafu)?;

    let mut relevant_hardware_case_ids = BTreeSet::<String>::new();

    for hardware_id_present in hardware_ids_present {
        match hardware_id_present {
            HardwareId::Pci(pci_id) => {
                if let Some(data) = pci_ids_to_hardware_case_ids_bucket.get(pci_id.into()) {
                    relevant_hardware_case_ids
                        .insert(String::from_utf8_lossy(data.kv().value()).to_string());
                }
            }
            HardwareId::Usb(usb_id) => {
                if let Some(data) = usb_ids_to_hardware_case_ids_bucket.get(usb_id.into()) {
                    relevant_hardware_case_ids
                        .insert(String::from_utf8_lossy(data.kv().value()).to_string());
                }
            }
        }
    }

    let mut relevant_driver_options = HashMap::<HardwareKind, BTreeSet<DriverOption>>::new();

    for relevant_hardware_case_id in relevant_hardware_case_ids {
        if let Some(data) =
            hardware_case_ids_to_driver_options_bucket.get(relevant_hardware_case_id.into())
        {
            let driver_option: DriverOption = rmp_serde::from_slice(data.kv().value()).unwrap();
            if 
        }
    }

    let mut process_hardware_listing_entry =
        |hardware_kind: &HardwareKind, driver_listing: &DriverListing| {
            for (hardware_ids, driver_records) in driver_listing.iter() {
                if !hardware_ids.is_disjoint(&hardware_ids_present) {
                    relevant_driver_records
                        .entry(hardware_kind.to_owned())
                        .or_default()
                        .extend(driver_records.clone().into_iter().filter(|driver_record| {
                            // println!("filter_tags: {:?}, tags: {:?}, driver_name: {}", filter_tags, driver_record.tags, driver_record.name);
                            filter_tags.is_empty() || !driver_record.tags.is_disjoint(&filter_tags)
                        }));
                }
            }
        };

    if let Some(hardware_kind) = optional_hardware {
        driver_database
            .read(|hardware_listing| {
                if let Some(driver_listing) = hardware_listing.get(&hardware_kind) {
                    process_hardware_listing_entry(&hardware_kind, driver_listing);
                }
            })
            .unwrap();
    } else {
        driver_database
            .read(|hardware_listing| {
                for (hardware_kind, driver_listing) in hardware_listing.iter() {
                    process_hardware_listing_entry(&hardware_kind, driver_listing);
                }
            })
            .unwrap();
    }

    Ok(relevant_driver_options)
}

pub fn search(search_action_arguments: SearchActionArguments) -> Result<SearchActionOutput, Error> {
    Ok(SearchActionOutput {
        inner: search_inner(
            search_action_arguments.database_file,
            search_action_arguments.hardware,
            search_action_arguments.tags,
        )?,
    })
}
