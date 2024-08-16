use std::process::Command;

use which::which;

use crate::utils::count_newlines_hyperscreaming;

pub fn get_packages() -> String {
    // FML
    // Thank FUCK neofetch is MIT or i would not bother
    let mut packages = "".to_string();

    if which("pacman").is_ok() {
        packages += format!("{} (pacman)", {
            let tmp = Command::new("pacman").arg("-Q").output();
            if tmp.is_ok() {
                count_newlines_hyperscreaming(String::from_utf8(tmp.unwrap().stdout.to_vec()).unwrap().as_str())
            } else {
                0
            }
        }).as_str();
    }

    return packages;
}