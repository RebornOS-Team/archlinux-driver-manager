use crate::error::{Error, PackageNotFoundSnafu};
use alpm::{Alpm, Package, TransFlag};
use alpm_utils::alpm_with_conf;
use pacmanconf::Config;

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
        packages_to_install: T,
        packages_to_remove: T,
    ) -> Result<(), Error> {
        let flags = TransFlag::NONE;
        self.handle.trans_init(flags).unwrap();

        let mut actual_install_list = Vec::<String>::new();
        let mut actual_remove_list = Vec::<String>::new();

        for package_name in packages_to_install {
            let package_name = package_name.as_ref();

            let package = self
                .handle
                .syncdbs()
                .iter()
                .find_map(|db| db.pkg(package_name).ok());

            if let Some(package) = package {
                self.handle.trans_add_pkg(package).unwrap();
                actual_install_list.push(package_name.to_owned());
            } else {
                self.handle.trans_release().unwrap();
                PackageNotFoundSnafu { name: package_name }.fail()?;
            }
        }

        for package_name in packages_to_remove {
            let package_name = package_name.as_ref();

            let package = self.get(package_name);

            if let Some(package) = package {
                self.handle.trans_remove_pkg(package).unwrap();
                actual_remove_list.push(package_name.to_owned());
            } else {
                self.handle.trans_release().unwrap();
                PackageNotFoundSnafu { name: package_name }.fail()?;
            }
        }

        self.handle.trans_prepare().unwrap();
        println!("Packages to Install: {:?}", actual_install_list);
        println!("Packages to Remove: {:?}", actual_remove_list);
        println!("Please wait while packages are being installed...");

        self.handle.trans_commit().unwrap();

        println!("Transaction completed.");

        Ok(())
    }
}
