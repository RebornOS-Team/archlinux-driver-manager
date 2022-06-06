use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(
        display("The driver database could not be opened. More details: {source}")
    )]
    Database {
        source: rustbreak::error::RustbreakError,
    },

    #[snafu(
        display("The argument \"{argument}\" is invalid. Permitted values are {{{allowed_arguments:#?}}}.")
    )]
    InvalidEnumArgument {
        argument: String,
        allowed_arguments: Vec<String>,
    },
}