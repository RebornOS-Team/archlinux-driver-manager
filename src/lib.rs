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

    pub struct CommandlineInterface {}

    impl CommandlineInterface {
        pub fn new() -> Self {
            CommandlineInterface {}
        }

        pub fn run(self) {
            let cli = Cli::parse();

            // match cli.command {
            //     Some(TextStorageServiceCommand::Privatebin(privatebin_arguments)) => {
            //         CommandlineInterface::privatebin(cli.arguments, cli.global_arguments).await;
            //     }
            //     Some(TextStorageServiceCommand::Dpaste {}) => {
            //         println!("DPaste...");
            //     }
            //     None => {
            //         CommandlineInterface::privatebin(cli.arguments, cli.global_arguments).await;
            //     }
            // }
        }
    }

    mod clap_template {
        use clap::{Args, Parser, Subcommand};

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

        #[derive(Debug, Subcommand)]
        #[clap(args_conflicts_with_subcommands = true)]
        pub enum ActionCommand {
            #[clap(
                about = "List installed drivers.",
                display_order = 1,
            )]
            List(ListActionArguments),

            #[clap(
                about = "Search for available drivers.",
                display_order = 2,
            )]
            Search(SearchActionArguments),

            #[clap(
                about = "Install Drivers.",
                display_order = 3,
            )]
            Install(InstallActionArguments),
        }

        #[derive(Debug, Args)]
        pub struct ListActionArguments {
            #[clap(
                help = "The hardware to list installed drivers for.",
                display_order = 11,
            )]
            pub hardware: Option<String>,

            #[clap(
                long = "tags",
                short = 't',
                help = "Tags to filter drivers.",
                display_order = 12,
            )]
            pub tags: Vec<String>,
        }

        #[derive(Debug, Args)]
        pub struct SearchActionArguments {
            #[clap(
                help = "The hardware to search drivers for.",
                display_order = 21,
            )]
            pub hardware: Option<String>,

            #[clap(
                long = "tags",
                short = 't',
                help = "Tags to filter drivers.",
                display_order = 22,
            )]
            pub tags: Vec<String>,
        }

        #[derive(Debug, Args)]
        pub struct InstallActionArguments {
            #[clap(
                help = "The hardware to install drivers for.",
                display_order = 31,
            )]
            pub hardware: Option<String>,

            #[clap(
                long = "tags",
                short = 't',
                help = "Tags to filter drivers.",
                display_order = 32,
            )]
            pub tags: Vec<String>,
        }
    }
}
