pub use commandline_interface_template::*;

use clap::Parser;
use serde::Serialize;
use crate::data::{DriverDatabase, DriverRecord, PciId};

pub struct CommandlineInterface {}

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

pub enum CommandlineOutputKind {
    Regular,
    Json,
    Plain,
    Debug,
}

impl CommandlineInterface {
    pub fn new() -> Self {
        CommandlineInterface {}
    }

    pub fn run(self) {
        let cli = Cli::parse();

        match cli.command {
            Some(ActionCommand::List(list_arguments)) => {}
            Some(ActionCommand::Search(search_arguments)) => {
                let db = DriverDatabase::try_new().unwrap();
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

pub mod commandline_interface_template {
    use clap::{Args, Parser, Subcommand};
    use std::path::PathBuf;

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

        #[clap(
            long = "database",
            help = "The `ron` database file to use for searching drivers.",
            display_order = 23
        )]
        pub database_file: Option<PathBuf>,
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

        #[clap(
            long = "database",
            help = "The `ron` database file to use for searching drivers.",
            display_order = 33
        )]
        pub database_file: Option<PathBuf>,
    }
}
