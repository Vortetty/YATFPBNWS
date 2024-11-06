use std::process::Command;
use sysinfo::{Pid, System};

pub fn get_shell(sys: &System) -> String {
    let shell_name = sys.process(Pid::from_u32(std::os::unix::process::parent_id()))
        .unwrap_or_else(|| sys.process(Pid::from_u32(std::process::id())).unwrap())
        .name()
        .to_string_lossy()
        .to_string()
        .to_lowercase();

    match shell_name.as_str() {
        "zsh" => {
            // This op is SO F_CKING SLOW OML
            // We need to optimize this somehow :/
            // update: found out this is actually fairly fast, it was sysinfo's threading that slowed it
            let tmp = Command::new("zsh").arg("--version").output();
            if tmp.is_ok() {
                format!("{}", String::from_utf8(tmp.unwrap().stdout.to_vec()).unwrap())
            } else {
                "zsh ?.?".to_string()
            }
        }
        _ => {
            format!("{} (unimplemented)", shell_name)
        }
    }
}
