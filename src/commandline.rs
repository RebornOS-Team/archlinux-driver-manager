pub use commandline_interface_template::*;

use crate::{
    actions::{generate_database, install, list, search},
    data::database::convert_tag,
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

impl CommandlineInterface {
    pub fn new() -> Self {
        CommandlineInterface {}
    }

    pub fn run(self) {
        let mut cli = Cli::parse();

        match cli.command {
            Some(ActionCommand::List(mut list_action_arguments)) => {
                list_action_arguments.tags =
                    list_action_arguments.tags.iter().map(convert_tag).collect();

                list::list(list_action_arguments).print_select(cli.global_arguments);
            }
            Some(ActionCommand::Search(mut search_action_arguments)) => {
                search_action_arguments.tags = search_action_arguments
                    .tags
                    .iter()
                    .map(convert_tag)
                    .collect();

                search::search(search_action_arguments).print_select(cli.global_arguments);
            }
            Some(ActionCommand::Install(mut install_action_arguments)) => {
                install_action_arguments.tags = install_action_arguments
                    .tags
                    .iter()
                    .map(convert_tag)
                    .collect();

                install::install(install_action_arguments).print_select(cli.global_arguments);
            }
            Some(ActionCommand::GenerateDatabase(generate_database_action_arguments)) => {
                generate_database::generate_database(generate_database_action_arguments)
                    .print_select(cli.global_arguments);
            }
            None => {
                cli.arguments.tags = cli.arguments.tags.iter().map(convert_tag).collect();

                list::list(cli.arguments).print_select(cli.global_arguments);
            }
        }
    }
}

pub mod commandline_interface_template {
    use clap::{Args, Parser, Subcommand};
    use std::path::PathBuf;

    use crate::data::database::HardwareKind;

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
        #[clap(name = "list", about = "List installed drivers.", display_order = 1)]
        List(ListActionArguments),

        #[clap(
            name = "search",
            about = "Search for available drivers.",
            display_order = 2
        )]
        Search(SearchActionArguments),

        #[clap(name = "install", about = "Install Drivers.", display_order = 3)]
        Install(InstallActionArguments),

        #[clap(
            name = "generate-database",
            alias = "gendb",
            about = "Generate database from input file.",
            display_order = 4
        )]
        GenerateDatabase(GenerateDatabaseActionArguments),
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
            long = "tag",
            short = 't',
            help = "Tags to filter drivers.",
            display_order = 12
        )]
        pub tags: Vec<String>,

        #[clap(
            long = "database",
            help = "Path to the `ron` database file to use for recognizing drivers.",
            default_value = "/var/lib/archlinux-driver-manager/database.ron",
            display_order = 13
        )]
        pub database_file: PathBuf,
    }

    #[derive(Debug, Args)]
    pub struct SearchActionArguments {
        #[clap(
            arg_enum,
            help = "The hardware to search drivers for.",
            display_order = 21
        )]
        pub hardware: Option<HardwareKind>,

        #[clap(
            long = "tag",
            short = 't',
            help = "Tags to filter drivers.",
            display_order = 22
        )]
        pub tags: Vec<String>,

        #[clap(
            long = "database",
            help = "Path to the `ron` database file to use for searching drivers.",
            default_value = "/var/lib/archlinux-driver-manager/database.ron",
            display_order = 23
        )]
        pub database_file: PathBuf,
    }

    #[derive(Debug, Args)]
    pub struct InstallActionArguments {
        #[clap(
            arg_enum,
            help = "The hardware to install drivers for.",
            display_order = 31
        )]
        pub hardware: HardwareKind,

        #[clap(
            long = "tag",
            short = 't',
            help = "Tags to filter drivers.",
            display_order = 32
        )]
        pub tags: Vec<String>,

        #[clap(
            long = "enable-aur",
            help = "Enable installing from the Arch User Repository (AUR).",
            display_order = 33
        )]
        pub enable_aur: bool,

        #[clap(
            long = "database",
            help = "Path to the `ron` database file to use for searching drivers.",
            default_value = "/var/lib/archlinux-driver-manager/database.ron",
            display_order = 34
        )]
        pub database_file: PathBuf,
    }

    #[derive(Debug, Args)]
    pub struct GenerateDatabaseActionArguments {
        #[clap(
            help = "Path to the input file (Only YAML is currently supported).",
            display_order = 41
        )]
        pub input_file: PathBuf,

        #[clap(
            help = "Path to the `ron` database file to generate.",
            default_value = "database.ron",
            display_order = 42
        )]
        pub database_file: PathBuf,
    }
}
