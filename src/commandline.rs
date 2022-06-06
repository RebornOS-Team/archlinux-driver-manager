pub use commandline_interface_template::*;

use crate::{
    actions::{install, list, search},
    data::{DriverDatabase, DriverRecord, PciId},
};
use clap::Parser;
use owo_colors::{OwoColorize, Stream::Stderr};
use std::fmt::Display;

pub struct CommandlineInterface {}

pub trait CommandlinePrint {
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
            Some(ActionCommand::List(list_action_arguments)) => {
                list::list(list_action_arguments).print_select(cli.global_arguments);
            }
            Some(ActionCommand::Search(search_action_arguments)) => {
                search::search(search_action_arguments).print_select(cli.global_arguments);
            }
            Some(ActionCommand::Install(install_action_arguments)) => {
                install::install(install_action_arguments).print_select(cli.global_arguments);
            }
            None => {
                list::list(cli.arguments).print_select(cli.global_arguments);
            }
        }
    }
}

impl<T, E> CommandlinePrint for Result<T, E>
where
    T: CommandlinePrint,
    E: Display,
{
    fn print(&self) {
        match self {
            Ok(inner) => inner.print(),
            Err(inner) => {
                let message = format!(
                    "{} {}",
                    "ERROR:".if_supports_color(Stderr, |text| text.red()),
                    inner,
                );
                eprintln!("{}", message);
            }
        }
    }
    fn print_json(&self) {
        match self {
            Ok(inner) => inner.print_json(),
            Err(inner) => {
                let message = format!(
                    "{} {}",
                    "ERROR:".if_supports_color(Stderr, |text| text.red()),
                    inner,
                );
                eprintln!("{}", message);
                println!("{{errors:[{}]}}", inner);
            }
        }
    }
    fn print_plain(&self) {
        match self {
            Ok(inner) => inner.print_plain(),
            Err(inner) => {
                let message = format!(
                    "{} {}",
                    "ERROR:".if_supports_color(Stderr, |text| text.red()),
                    inner,
                );
                eprintln!("{}", message);
                println!("");
            }
        }
    }
    fn print_debug(&self) {
        match self {
            Ok(inner) => inner.print_debug(),
            Err(inner) => {
                let message = format!(
                    "{} {}",
                    "ERROR:".if_supports_color(Stderr, |text| text.red()),
                    inner,
                );
                eprintln!("{}", message);
            }
        }
    }
}

pub mod commandline_interface_template {
    use clap::{Args, Parser, Subcommand};
    use std::path::PathBuf;

    use crate::data::HardwareKind;

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
            arg_enum,
            help = "The hardware to list installed drivers for.",
            display_order = 11
        )]
        pub hardware: Option<HardwareKind>,

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
        #[clap(
            arg_enum,
            help = "The hardware to search drivers for.",
            display_order = 21)]
        pub hardware: Option<HardwareKind>,

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
        #[clap(
            arg_enum,
            help = "The hardware to install drivers for.",
            display_order = 31)]
        pub hardware: HardwareKind,

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
