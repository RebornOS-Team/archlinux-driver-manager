use crate::data::input_file::HardwareKind;
use crate::{
    cli::{CommandlinePrint, SearchActionArguments},
    data::database::DriverDatabase,
    data::{
        database::{HardwareId, PciId, UsbId},
        input_file::{DriverOption, HardwareSetup},
    },
    error::{DatabaseSnafu, Error},
};
use devices;
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SearchActionOutput {
    inner: BTreeMap<HardwareKind, BTreeSet<DriverOption>>,
}

impl SearchActionOutput {
    pub fn new() -> Self {
        SearchActionOutput {
            inner: BTreeMap::<HardwareKind, BTreeSet<DriverOption>>::new(),
        }
    }
}

impl Deref for SearchActionOutput {
    type Target = BTreeMap<HardwareKind, BTreeSet<DriverOption>>;

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
    devices::Devices::get()
        .expect("Failed to get connected devices")
        .into_iter()
        .map(|item| match item.path() {
            devices::DevicePath::PCI {
                bus: _,
                slot: _,
                function: _,
            } => HardwareId::Pci(PciId {
                vendor: item.vendor_id(),
                device: item.product_id(),
            }),
            devices::DevicePath::USB { bus: _, device: _ } => HardwareId::Usb(UsbId {
                vendor: item.vendor_id(),
                device: item.product_id(),
            }),
        })
        .collect()
}

pub fn search_inner<T: Iterator<Item = String>>(
    database_filepath: PathBuf,
    optional_hardware: Option<&HardwareKind>,
    tags: T,
) -> Result<BTreeMap<HardwareKind, BTreeSet<DriverOption>>, Error> {
    let driver_database = DriverDatabase::with_database_path(database_filepath)?;

    // Open a read-only transaction to get the data
    let transaction = driver_database.tx(false).context(DatabaseSnafu {})?;

    let filter_tags: BTreeSet<String> = tags.into_iter().collect();

    let hardware_kind_to_hardware_setup_id_bucket = transaction
        .get_bucket("hardware_kind_to_hardware_setup_id_bucket")
        .context(DatabaseSnafu)?;

    let hardware_setup_id_to_hardware_setup_bucket = transaction
        .get_bucket("hardware_setup_id_to_hardware_setup_bucket")
        .context(DatabaseSnafu)?;

    let hardware_ids_present = hardware_ids_present();

    let result = BTreeMap::<HardwareKind, BTreeSet<DriverOption>>::new();

    if let Some(hardware_kind) = optional_hardware {
        if let Some(data) = hardware_kind_to_hardware_setup_id_bucket.get(hardware_kind.to_string())
        {
            let hardware_setup_ids: BTreeSet<String> =
                rmp_serde::from_slice(data.kv().value()).unwrap();
            return Ok(hardware_setup_ids
                .iter()
                .filter_map(|hardware_setup_id| {
                    if let Some(hardware_setup_data) =
                        hardware_setup_id_to_hardware_setup_bucket.get(hardware_setup_id)
                    {
                        rmp_serde::from_slice(hardware_setup_data.kv().value()).ok()
                    } else {
                        None
                    }
                })
                .fold(
                    BTreeMap::<HardwareKind, BTreeSet<DriverOption>>::new(),
                    |mut grouped_driver_options, hardware_setup: HardwareSetup| {
                        if let Some(more_driver_options) = hardware_setup.matching_driver_options(
                            hardware_ids_present,
                            optional_hardware,
                            &tags,
                        ) {
                            grouped_driver_options
                                .entry(hardware_setup.hardware_kind)
                                .or_default()
                                .extend(more_driver_options.into_iter().map(|item| *item));
                        }
                        grouped_driver_options
                    },
                ));
        } else {
            return Ok(BTreeMap::<HardwareKind, BTreeSet<DriverOption>>::new());
        }
    } else {
        return Ok(hardware_setup_id_to_hardware_setup_bucket
            .kv_pairs()
            .filter_map(|data| rmp_serde::from_slice(data.value()).ok())
            .fold(
                BTreeMap::<HardwareKind, BTreeSet<DriverOption>>::new(),
                |mut grouped_driver_options, hardware_setup: HardwareSetup| {
                    if let Some(more_driver_options) = hardware_setup.matching_driver_options(
                        hardware_ids_present,
                        optional_hardware,
                        &tags,
                    ) {
                        grouped_driver_options
                            .entry(hardware_setup.hardware_kind)
                            .or_default()
                            .extend(more_driver_options.into_iter().map(|item| *item));
                    }
                    grouped_driver_options
                },
            ));
    }
}

pub fn search(search_action_arguments: SearchActionArguments) -> Result<SearchActionOutput, Error> {
    Ok(SearchActionOutput {
        inner: search_inner(
            search_action_arguments.database_file,
            search_action_arguments.hardware.as_ref(),
            search_action_arguments.tags.into_iter(),
        )?,
    })
}
