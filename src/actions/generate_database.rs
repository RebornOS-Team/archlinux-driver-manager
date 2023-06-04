use crate::{
    cli::{CommandlinePrint, GenerateDatabaseActionArguments},
    data::{
        database,
        input_file::{self, HardwareList, HardwareListInner, PciIdList, UsbIdList},
    },
    error::{DatabaseSnafu, Error},
};
use owo_colors::{OwoColorize, Stream::Stdout};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use speedy::{Readable, Writable};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

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
    let hardware_setups = input_file::parse_input_file(input_file)?;
    let driver_database = database::DriverDatabase::with_database_path(database_file)?;

    // open a writable transaction so we can make changes
    let transaction = driver_database.tx(true).context(DatabaseSnafu)?;

    let pci_id_to_hardware_setup_id_bucket = transaction
        .get_or_create_bucket("pci_id_to_hardware_setup_id_bucket")
        .context(DatabaseSnafu)?;

    let usb_id_to_hardware_setup_id_bucket = transaction
        .get_or_create_bucket("usb_id_to_hardware_setup_id_bucket")
        .context(DatabaseSnafu)?;

    let hardware_kind_to_hardware_setup_id_bucket = transaction
        .get_or_create_bucket("hardware_kind_to_hardware_setup_id_bucket")
        .context(DatabaseSnafu)?;

    let hardware_kind_to_driver_option_id_bucket = transaction
        .get_or_create_bucket("hardware_kind_to_driver_option_id_bucket")
        .context(DatabaseSnafu)?;

    let hardware_setup_id_to_driver_option_id_bucket = transaction
        .get_or_create_bucket("hardware_setup_id_to_driver_option_id_bucket")
        .context(DatabaseSnafu)?;

    let hardware_setup_id_to_hardware_setup_bucket = transaction
        .get_or_create_bucket("hardware_setup_id_to_hardware_setup_bucket")
        .context(DatabaseSnafu)?;

    let driver_option_id_to_driver_option_bucket = transaction
        .get_or_create_bucket("driver_option_id_to_driver_option_bucket")
        .context(DatabaseSnafu)?;

    static HARDWARE_SETUP_SERIAL: AtomicUsize = AtomicUsize::new(1);
    let new_hardware_setup_id = || {
        HARDWARE_SETUP_SERIAL
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
    };

    static DRIVER_OPTION_SERIAL: AtomicUsize = AtomicUsize::new(1);
    let new_driver_option_id = || {
        DRIVER_OPTION_SERIAL
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
    };

    hardware_setups.iter().for_each(|hardware_setup| {
        let hardware_setup_id = new_hardware_setup_id();
        let mut driver_option_ids = BTreeSet::<String>::new();

        {
            let mut hardware_setup_ids = BTreeSet::<String>::new();
            if let Some(data) = hardware_kind_to_hardware_setup_id_bucket
                .get(hardware_setup.hardware_kind.to_string())
            {
                if data.is_kv() {
                    let kv = data.kv();
                    hardware_setup_ids = BTreeSet::<String>::read_from_buffer(kv.value()).unwrap();
                }
            }
            hardware_setup_ids.insert(hardware_setup_id.clone());
            hardware_kind_to_hardware_setup_id_bucket
                .put(
                    hardware_setup.hardware_kind.to_string(),
                    hardware_setup_ids.write_to_vec().unwrap(),
                )
                .context(DatabaseSnafu)
                .unwrap();
        }

        hardware_setup_id_to_hardware_setup_bucket
            .put(
                hardware_setup_id.clone(),
                hardware_setup.write_to_vec().unwrap(),
            )
            .context(DatabaseSnafu)
            .unwrap();

        let process_pci_id_list = |pci_id_list: &PciIdList| {
            pci_id_list.devices.iter().for_each(|device| {
                let pci_id = (((pci_id_list.vendor as u32) << 16) | (*device as u32)).to_string();
                let mut hardware_setup_ids = BTreeSet::<String>::new();
                if let Some(data) = pci_id_to_hardware_setup_id_bucket.get(&pci_id) {
                    if data.is_kv() {
                        let kv = data.kv();
                        hardware_setup_ids =
                            BTreeSet::<String>::read_from_buffer(kv.value()).unwrap();
                    }
                }
                hardware_setup_ids.insert(hardware_setup_id.clone());
                pci_id_to_hardware_setup_id_bucket
                    .put(pci_id, hardware_setup_ids.write_to_vec().unwrap())
                    .context(DatabaseSnafu)
                    .unwrap();
            })
        };

        let process_usb_id_list = |usb_id_list: &UsbIdList| {
            usb_id_list.devices.iter().for_each(|device| {
                let usb_id = (((usb_id_list.vendor as u32) << 16) | (*device as u32)).to_string();
                let mut hardware_setup_ids = BTreeSet::<String>::new();
                if let Some(data) = usb_id_to_hardware_setup_id_bucket.get(&usb_id) {
                    if data.is_kv() {
                        let kv = data.kv();
                        hardware_setup_ids =
                            BTreeSet::<String>::read_from_buffer(kv.value()).unwrap();
                    }
                }
                hardware_setup_ids.insert(hardware_setup_id.clone());
                usb_id_to_hardware_setup_id_bucket
                    .put(usb_id, hardware_setup_ids.write_to_vec().unwrap())
                    .context(DatabaseSnafu)
                    .unwrap();
            })
        };

        match &hardware_setup.hardware_list {
            HardwareList::Each(hardware_lists) => {
                hardware_lists
                    .iter()
                    .for_each(|hardware_list_inner| match hardware_list_inner {
                        HardwareListInner::Pci(pci_id_list) => process_pci_id_list(pci_id_list),
                        HardwareListInner::Usb(usb_id_list) => process_usb_id_list(usb_id_list),
                    })
            }
            HardwareList::Pci(pci_id_list) => process_pci_id_list(&pci_id_list),
            HardwareList::Usb(usb_id_list) => process_usb_id_list(&usb_id_list),
        }

        hardware_setup
            .driver_options
            .iter()
            .for_each(|driver_option| {
                let driver_option_id = new_driver_option_id();

                {
                    let mut driver_option_ids = BTreeSet::<String>::new();
                    if let Some(data) = hardware_kind_to_driver_option_id_bucket
                        .get(hardware_setup.hardware_kind.to_string())
                    {
                        if data.is_kv() {
                            let kv = data.kv();
                            driver_option_ids =
                                BTreeSet::<String>::read_from_buffer(kv.value()).unwrap();
                        }
                    }
                    driver_option_ids.insert(driver_option_id.clone());
                    hardware_kind_to_driver_option_id_bucket
                        .put(
                            hardware_setup.hardware_kind.to_string(),
                            driver_option_ids.write_to_vec().unwrap(),
                        )
                        .context(DatabaseSnafu)
                        .unwrap();
                }

                driver_option_ids.insert(driver_option_id.clone());
                driver_option_id_to_driver_option_bucket
                    .put(driver_option_id, driver_option.write_to_vec().unwrap())
                    .context(DatabaseSnafu)
                    .unwrap();
            });

        hardware_setup_id_to_driver_option_id_bucket
            .put(hardware_setup_id, driver_option_ids.write_to_vec().unwrap())
            .context(DatabaseSnafu)
            .unwrap();
    });

    transaction.commit().context(DatabaseSnafu)?;

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
