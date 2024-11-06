use std::process::Command;
use sysinfo::{Pid, System};
use regex::{self, Regex};

pub fn get_term(sys: &System) -> String {
    let shell_name = sys.process(
            sys.process(Pid::from_u32(std::os::unix::process::parent_id()))
            .unwrap_or_else(|| sys.process(Pid::from_u32(std::process::id())).unwrap())
            .parent().unwrap_or_else(|| Pid::from_u32(std::process::id()))
        )
        .unwrap_or_else(|| sys.process(Pid::from_u32(std::process::id())).unwrap())
        .name()
        .to_string_lossy()
        .to_string()
        .to_lowercase();

    match shell_name.as_str() {
        "kitty" => {
            let tmp = Command::new("kitty").arg("--version").output();
            if tmp.is_ok() {
                let version_pattern = Regex::new(r"\d+\.\d+\.\d+").ok().unwrap();
                format!("Kitty {}", version_pattern.find(String::from_utf8(tmp.unwrap().stdout.to_vec()).unwrap().as_str()).map(|m| m.as_str()).unwrap())
            } else {
                "Kitty".to_string()
            }
        }
        _ => {
            format!("{} (unimplemented)", shell_name)
        }
    }
}
