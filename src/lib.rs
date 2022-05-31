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

    use clap::Parser;

    pub use clap_template::*;
    use serde::Serialize;

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
                Some(ActionCommand::Search(search_arguments)) => {}
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
    use rangemap::RangeInclusiveMap;
    use serde::{Serialize, Deserialize};
    use std::{path::PathBuf, collections::HashMap};

    pub type DriverListing = RangeInclusiveMap<u32, Vec<DriverRecord>>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DriverRecord {
        pub packages: Vec<String>,
        pub configs: Vec<ConfigRecord>,
        pub tags: Vec<String>,
        pub pre_install_script: Option<PathBuf>,
        pub post_install_script: Option<PathBuf>,        
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConfigRecord {
        pub format: ConfigFormat,
        pub path: Option<PathBuf>,
        pub entries: HashMap<String, String>,    
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ConfigFormat {
        Ini,
        Json,
        Yaml,
        Toml,
        Xml,    
    }
}
