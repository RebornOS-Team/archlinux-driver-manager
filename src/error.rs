use std::path::PathBuf;

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
        display("The input file at {} could not be parsed for driver data. More details: {}", path.to_string_lossy(), source)
    )]
    InputFileParse {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[snafu(
        display("The value {value} could not be converted to the enumeration {enum_name}. The allowed values are {allowed_values:?}")
    )]
    EnumValue {
        value: String,
        enum_name: String,
        allowed_values: Vec<String>,
    }
}