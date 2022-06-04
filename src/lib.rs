pub use data::*;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "The provided value \"{}\" for the parameter \"{}\" is invalid. The supported values are: {:?}", value , parameter_name , supported_values
    ))]
    EnumParameterFormatError {
        value: String,
        parameter_name: String,
        supported_values: Vec<String>,
    },

    #[snafu(display(
        "The provided value \"{}\" for the parameter \"{}\" is invalid. The correct format is: {}",
        value,
        parameter_name,
        supported_format
    ))]
    StringParameterFormatError {
        value: String,
        parameter_name: String,
        supported_format: String,
    },
}

pub mod commandline {

    pub use clap_template::*;

    use crate::*;
    use clap::Parser;
    use serde::Serialize;
    use std::ops::RangeInclusive;

    pub enum CommandlineOutputKind {
        Regular,
        Json,
        Plain,
        Debug,
    }

    pub trait CommandlineFlags {
        fn json_flag(&self) -> bool;
        fn plain_flag(&self) -> bool;
        fn debug_flag(&self) -> bool;
        fn output_kind(&self) -> CommandlineOutputKind {
            if self.json_flag() {
                return CommandlineOutputKind::Json;
            } else if self.plain_flag() {
                return CommandlineOutputKind::Plain;
            } else if self.debug_flag() {
                return CommandlineOutputKind::Debug;
            } else {
                return CommandlineOutputKind::Regular;
            }
        }
    }

    pub trait CommandlinePrint: Serialize {
        fn print(&self);
        fn print_json(&self);
        fn print_plain(&self);
        fn print_debug(&self);
        fn print_select(&self, flags: impl CommandlineFlags) {
            match CommandlineFlags::output_kind(&flags) {
                CommandlineOutputKind::Regular => self.print(),
                CommandlineOutputKind::Json => self.print_json(),
                CommandlineOutputKind::Plain => self.print_plain(),
                CommandlineOutputKind::Debug => self.print_debug(),
            }
        }
    }

    pub struct CommandlineInterface {}

    impl CommandlineInterface {
        pub fn new() -> Self {
            CommandlineInterface {}
        }

        pub fn run(self) {
            let cli = Cli::parse();

            match cli.command {
                Some(ActionCommand::List(list_arguments)) => {}
                Some(ActionCommand::Search(search_arguments)) => {
                    let db = DriverDatabase::load_from_path_or_default("test.ron").unwrap();
                    println!("Writing to Database");
                    db.write(|db| {
                        db.insert(
                            PciId::range_inclusive("1234:5678", "1234:56ab")
                                .expect("Invalid PCI IDs supplied"),
                            vec![DriverRecord::default()],
                        );
                        println!("Entries: \n{:#?}", db);
                    })
                    .unwrap();

                    println!("Syncing Database");
                    db.save().unwrap();

                    println!("Loading Database");
                    db.load().unwrap();

                    println!("Reading from Database");
                    db.read(|db| {
                        println!("Results:");
                        println!("{:#?}", db);
                    })
                    .unwrap();
                }
                Some(ActionCommand::Install(install_arguments)) => {}
                None => {}
            }
        }
    }

    mod clap_template {
        use clap::{Args, Parser, Subcommand};

        use super::CommandlineFlags;

        #[derive(Debug, Parser)]
        #[clap(version, author, about, args_conflicts_with_subcommands = true)]
        pub struct Cli {
            #[clap(flatten)]
            pub global_arguments: GlobalArguments,

            #[clap(subcommand)]
            pub command: Option<ActionCommand>,

            #[clap(flatten)]
            pub arguments: ListActionArguments,
        }

        #[derive(Debug, Args)]
        #[clap(args_conflicts_with_subcommands = true)]
        pub struct GlobalArguments {
            #[clap(
                long = "json",
                help = "Output in the JSON format for machine readability and scripting purposes.",
                takes_value = false,
                global = true,
                display_order = usize::MAX - 3,
            )]
            pub json_flag: bool,

            #[clap(
                long = "plain",
                help = "Output as plain text without extra information, for machine readability and scripting purposes.",
                takes_value = false,
                global = true,
                display_order = usize::MAX - 2,
            )]
            pub plain_flag: bool,

            #[clap(
                long = "debug",
                help = "Output debug messages.",
                takes_value = false,
                global = true,
                display_order = usize::MAX - 1,
            )]
            pub debug_flag: bool,
        }

        impl CommandlineFlags for GlobalArguments {
            fn json_flag(&self) -> bool {
                return self.json_flag;
            }

            fn plain_flag(&self) -> bool {
                return self.plain_flag;
            }

            fn debug_flag(&self) -> bool {
                return self.debug_flag;
            }
        }

        #[derive(Debug, Subcommand)]
        #[clap(args_conflicts_with_subcommands = true)]
        pub enum ActionCommand {
            #[clap(about = "List installed drivers.", display_order = 1)]
            List(ListActionArguments),

            #[clap(about = "Search for available drivers.", display_order = 2)]
            Search(SearchActionArguments),

            #[clap(about = "Install Drivers.", display_order = 3)]
            Install(InstallActionArguments),
        }

        #[derive(Debug, Args)]
        pub struct ListActionArguments {
            #[clap(
                help = "The hardware to list installed drivers for.",
                display_order = 11
            )]
            pub hardware: Option<String>,

            #[clap(
                long = "tags",
                short = 't',
                help = "Tags to filter drivers.",
                display_order = 12
            )]
            pub tags: Vec<String>,
        }

        #[derive(Debug, Args)]
        pub struct SearchActionArguments {
            #[clap(help = "The hardware to search drivers for.", display_order = 21)]
            pub hardware: Option<String>,

            #[clap(
                long = "tags",
                short = 't',
                help = "Tags to filter drivers.",
                display_order = 22
            )]
            pub tags: Vec<String>,
        }

        #[derive(Debug, Args)]
        pub struct InstallActionArguments {
            #[clap(help = "The hardware to install drivers for.", display_order = 31)]
            pub hardware: Option<String>,

            #[clap(
                long = "tags",
                short = 't',
                help = "Tags to filter drivers.",
                display_order = 32
            )]
            pub tags: Vec<String>,
        }
    }
}

pub mod data {
    use rangemap::{RangeInclusiveMap, StepLite};
    use rustbreak::{deser::Ron, FileDatabase};
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use std::num::ParseIntError;
    use std::{
        collections::HashMap,
        fmt::{Debug, Display},
        ops::{Range, RangeInclusive},
        path::PathBuf,
        str::FromStr,
    };

    // pub type DriverListing = RangeInclusiveMap<u32, Vec<DriverRecord>>;
    pub type DriverListing = RangeInclusiveMap<PciId, Vec<DriverRecord>>;
    pub type DriverDatabase = FileDatabase<DriverListing, Ron>;

    #[derive(
        Default,
        PartialEq,
        Eq,
        PartialOrd, // Required by Ord
        Ord,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Copy,
        Clone, // Required by RangeInclusiveMap to implement Serialize and Deserialize
    )]
    pub struct PciId {
        value: u32,
    }

    impl PciId {
        pub fn new(vendor_id: u16, device_id: u16) -> Self {
            Self {
                value: (vendor_id as u32) * 16u32.pow(4) + (device_id as u32),
            }
        }

        pub fn vendor_id(&self) -> u16 {
            let vendor_id = self.value / 16u32.pow(4);
            println!(
                "self.value: {:08x}, vendor_id: {:04x}",
                self.value, vendor_id
            );
            vendor_id
                .try_into()
                .expect("The Vendor ID does not fit into an unsigned 16-bit integer.")
        }

        pub fn device_id(&self) -> u16 {
            let device_id = self.value % 16u32.pow(4);
            println!(
                "self.value: {:08x}, device_id: {:04x}",
                self.value, device_id
            );
            device_id
                .try_into()
                .expect("The Device ID does not fit into an unsigned 16-bit integer.")
        }

        pub fn range(start: &str, end: &str) -> Result<Range<Self>, ParsePciIdError> {
            Ok(Range {
                start: start.parse()?,
                end: end.parse()?,
            })
        }

        pub fn range_inclusive(
            start: &str,
            end: &str,
        ) -> Result<RangeInclusive<Self>, ParsePciIdError> {
            Ok(RangeInclusive::new(start.parse()?, end.parse()?))
        }
    }

    impl Display for PciId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:04x}:{:04x}", self.vendor_id(), self.device_id())
        }
    }

    impl Debug for PciId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PciId")
                .field("vendor_id", &format!("{:04x}", &self.vendor_id()))
                .field("device_id", &format!("{:04x}", &self.device_id()))
                .finish()
        }
    }

    impl Serialize for PciId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&format!("{}", self))
        }
    }

    impl<'de> Deserialize<'de> for PciId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Visitor;
            impl<'de> de::Visitor<'de> for Visitor {
                type Value = PciId;
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "a PCI ID")
                }
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    v.parse().map_err(E::custom)
                }
            }
            deserializer.deserialize_str(Visitor)
        }
    }

    impl StepLite for PciId {
        fn add_one(&self) -> Self {
            Self {
                value: self.value + 1,
            }
        }

        fn sub_one(&self) -> Self {
            Self {
                value: self.value - 1,
            }
        }
    }

    impl FromStr for PciId {
        type Err = ParsePciIdError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (vendor_id, device_id) = s.split_once(':').ok_or(ParsePciIdError::MissingColon)?;
            let vendor_id = u16::from_str_radix(vendor_id, 16)
                .map_err(|parse_int_error| ParsePciIdError::InvalidVendorId(parse_int_error))?;
            let device_id = u16::from_str_radix(device_id, 16)
                .map_err(|parse_int_error| ParsePciIdError::InvalidDeviceId(parse_int_error))?;
            Ok(Self::new(vendor_id, device_id))
        }
    }

    #[derive(Clone, Debug)]
    pub enum ParsePciIdError {
        InvalidVendorId(ParseIntError),
        InvalidDeviceId(ParseIntError),
        MissingColon,
    }

    impl Display for ParsePciIdError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ParsePciIdError::InvalidVendorId(parse_int_error) => {
                    write!(f, "Invalid Vendor ID. Please refer to {}", parse_int_error)
                }
                ParsePciIdError::InvalidDeviceId(parse_int_error) => {
                    write!(f, "Invalid Device ID. Please refer to {}", parse_int_error)
                }
                ParsePciIdError::MissingColon => {
                    write!(f, "Invalid PCI ID. Please ensure that the Vendor and Device IDs are separated by a colon `:`")
                }
            }
        }
    }

    #[derive(
        Default,
        Debug,
        PartialEq, // Required to implement Eq
        Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Serialize,
        Deserialize,
    )]
    pub struct DriverRecord {
        pub packages: Vec<String>,
        pub configs: Vec<ConfigRecord>,
        pub tags: Vec<String>,
        pub pre_install_script: Option<Script>,
        pub post_install_script: Option<Script>,
    }

    #[derive(
        Default,
        Debug,
        PartialEq, // Required to implement Eq
        Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Serialize,
        Deserialize,
    )]
    pub struct Script {
        pub script_kind: ScriptKind,
        pub path: Option<PathBuf>,
    }

    #[derive(
        Debug,
        PartialEq, // Required to implement Eq
        Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Serialize,
        Deserialize,
    )]
    pub enum ScriptKind {
        Python,
        JavaScript,
        Shell,
    }

    impl Default for ScriptKind {
        fn default() -> Self {
            return Self::Shell;
        }
    }

    #[derive(
        Default,
        Debug,
        PartialEq, // Required to implement Eq
        Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Serialize,
        Deserialize,
    )]
    pub struct ConfigRecord {
        pub format: ConfigFormat,
        pub path: Option<PathBuf>,
        pub entries: HashMap<String, String>,
    }

    #[derive(
        Debug,
        PartialEq, // Required to implement Eq
        Eq,        // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Clone,     // Required by RangeInclusiveMap to implement Serialize and Deserialize
        Serialize,
        Deserialize,
    )]
    pub enum ConfigFormat {
        Ini,
        Json,
        Yaml,
        Toml,
        Xml,
    }

    impl Default for ConfigFormat {
        fn default() -> Self {
            return ConfigFormat::Ini;
        }
    }
}
