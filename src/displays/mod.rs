use std::fs::{self, read_dir};
use regex::Regex;

pub fn get_displays() -> String {
    let output_pattern = Regex::new(r"(?i)^card\d+-[A-Z59_-]+?-\d+$").unwrap();

    let mut monitors: Vec<String> = vec![];

    // Iterate over DRM devices in sysfs
    for entry in read_dir("/sys/class/drm").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let filename = entry.file_name().into_string().unwrap();

        // Check if the directory matches the output pattern and has a `status` file
        if output_pattern.is_match(&filename) && path.join("status").is_file() && path.join("edid").is_file() {
            // Read the connection status of the display
            let status = fs::read_to_string(path.join("status")).unwrap();
            if status.trim() == "connected" {
                let modes_path = path.join("modes");
                if let Ok(modes) = fs::read_to_string(modes_path) {
                    // The first line usually contains the maximum resolution
                    if let Some(max_resolution) = modes.lines().next() {
                        //println!("Max resolution: {}", max_resolution);
                        monitors.push(max_resolution.to_string());
                    }
                }
            }
        }
    };

    if monitors.len() == 0 {
        return "None found".to_string();
    } else if monitors.len() == 1 {
        return "╰ ".to_string() + monitors[0].as_str();
    } else {
        let mut out = "".to_string();
        for (i, mon) in monitors.iter().enumerate() {
            if i == monitors.len()-1 {
                out.push_str("\n╰ ");
            } else {
                out.push_str("\n│ ");
            }
            out.push_str(mon.as_str());
        }
        return out.trim().to_string();
    }
}
