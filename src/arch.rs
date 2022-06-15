use alpm::{Alpm, Package, TransFlag};
use alpm_utils::alpm_with_conf;
use pacmanconf::Config;
use crate::error::{Error, PackageNotFoundSnafu};

pub const PACMAN_CONFIG_PATH: &str = "/etc/pacman.conf";

pub struct PackageManager {
    handle: Alpm,
}

impl PackageManager {
    pub fn new() -> Self {
        let pacman_conf = Config::from_file(PACMAN_CONFIG_PATH).unwrap();
        let alpm_handle = alpm_with_conf(&pacman_conf).unwrap();
        Self {
            handle: alpm_handle,
        }
    }

    pub fn get<S: AsRef<str>>(&self, package_name: S) -> Option<Package> {
        let db = self.handle.localdb();
        db.pkg(package_name.as_ref()).ok()
    }

    pub fn install<S: AsRef<str>, T: IntoIterator<Item = S>>(
        &mut self,
        package_names: T,
    ) -> Result<(), Error> {
        let flags = TransFlag::NONE;
        self.handle.trans_init(flags).unwrap();

        for package_name in package_names {
            let package_name = package_name.as_ref();

            let package = self
                .handle
                .syncdbs()
                .iter()
                .find_map(|db| db.pkg(package_name).ok());

            if let Some(package) = package {
                self.handle.trans_add_pkg(package).unwrap();                
            } else {
                self.handle.trans_release().unwrap();
                PackageNotFoundSnafu {
                    name: package_name,
                }.fail()?;
            }
        }

        self.handle.trans_prepare().unwrap();
        println!("Installing: {:#?}", self.handle.trans_add());

        self.handle.trans_commit().unwrap();

        println!("Transaction completed...");

        Ok(())
    }
}
