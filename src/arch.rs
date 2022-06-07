use alpm::{Alpm, Package};
use pacmanconf::Config;
use alpm_utils::alpm_with_conf;

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
}
