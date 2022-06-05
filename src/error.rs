use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(
        display("The driver database could not be opened. More details: {source}")
    )]
    Database {
        source: rustbreak::error::RustbreakError,
    }
}