use std::{fs, path::Path};

pub fn get_model() -> String {
    let mut name: String = String::new();

    if Path::new("/sys/devices/virtual/dmi/id/board_vendor").is_file()
        || Path::new("/sys/devices/virtual/dmi/id/board_name").is_file()
    {
        if Path::new("/sys/devices/virtual/dmi/id/board_vendor").is_file() {
            name += &fs::read_to_string("/sys/devices/virtual/dmi/id/board_vendor")
                .expect("Failed to read \"/sys/devices/virtual/dmi/id/board_vendor\"").trim_end();
        }
        if Path::new("/sys/devices/virtual/dmi/id/board_name").is_file() {
            name += " ";
            name += &fs::read_to_string("/sys/devices/virtual/dmi/id/board_name")
                .expect("Failed to read \"/sys/devices/virtual/dmi/id/board_name\"").trim_end();
        }
    } else if Path::new("/sys/devices/virtual/dmi/id/product_name").is_file()
        || Path::new("/sys/devices/virtual/dmi/id/product_version").is_file()
    {
        if Path::new("/sys/devices/virtual/dmi/id/product_name").is_file() {
            name += &fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
                .expect("Failed to read \"/sys/devices/virtual/dmi/id/product_name\"").trim_end();
        }
        if Path::new("/sys/devices/virtual/dmi/id/product_version").is_file() {
            name += " ";
            name += &fs::read_to_string("/sys/devices/virtual/dmi/id/product_version")
                .expect("Failed to read \"/sys/devices/virtual/dmi/id/product_version\"").trim_end();
        }
    } else if Path::new("/sys/firmware/devicetree/base/model").is_file() {
        name += &fs::read_to_string("/sys/firmware/devicetree/base/model")
            .expect("Failed to read \"/sys/firmware/devicetree/base/model\"").trim_end();
    } else if Path::new("/tmp/sysinfo/model").is_file() {
        name += &fs::read_to_string("/tmp/sysinfo/model")
            .expect("Failed to read \"/tmp/sysinfo/model\"").trim_end();
    }

    return name;
}
