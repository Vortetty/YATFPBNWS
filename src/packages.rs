use std::fs;

use which::which;

//use crate::utils::count_newlines_hyperscreaming;

pub fn get_packages() -> String {
    let mut packages = "".to_string();

    if which("pacman").is_ok() {
        let cnt = fs::read_dir("/var/lib/pacman/local/").unwrap().count() - 1;
        if cnt > 0 {
            packages += format!("{} (pacman), ", cnt).as_str()
        }
    }

    return packages.trim_end().to_string().trim_end_matches(",").to_string().trim_end_matches(", ").to_string();
}