pub mod commandline;
pub mod data;

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
