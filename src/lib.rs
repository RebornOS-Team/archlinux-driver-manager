use lazy_static::lazy_static;

lazy_static! {
    pub static ref DB_PATH: &'static str = "/var/lib/archlinux-driver-manager/database.db";
    pub static ref DB_PATH_TEMP: &'static str = "/tmp/archlinux-driver-manager/database.db";
}

pub mod actions;
pub mod arch;
pub mod cli;
pub mod data;
pub mod error;
