use std::{env, process::Command};
use detect_desktop_environment::DesktopEnvironment;
use regex::Regex;

pub fn get_de() -> String {
    let de = DesktopEnvironment::detect();
    let mut detext = "Unknown".to_string();
    if de.is_some() {
        match de.unwrap() {
            DesktopEnvironment::Kde => {
                let tmp = Command::new("plasmashell").arg("--version").output();
                detext = if tmp.is_ok() {
                    let version_pattern = Regex::new(r"\d+\.\d+\.\d+").ok().unwrap();
                    format!("Kde {}", version_pattern.find(String::from_utf8(tmp.unwrap().stdout.to_vec()).unwrap().as_str()).map(|m| m.as_str()).unwrap())
                } else {
                    "Kde".to_string()
                }
            }
            _ => {
                detext = format!("{:?}", de.unwrap());
            }
        }
    }
    let x11_display = env::var("DISPLAY");
    let wayland_display = env::var("WAYLAND_DISPLAY");
    if wayland_display.is_ok() && x11_display.is_ok() {
        detext += " (Wayland + XWayland)";
    } else if wayland_display.is_ok() {
        detext += " (Wayland)";
    } else if x11_display.is_ok() {
        detext += " (X11)";
    } else {
        detext += " (Unknown)";
    }

    return detext;
}