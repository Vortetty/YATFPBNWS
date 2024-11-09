use std::collections::HashMap;

use regex::Regex;
use wgpu::Adapter;

struct AdapterCnt {
    count: i64,
    name: String,
    drivers: Vec<String>,
    vulkan: bool,
    gl: bool,
    unrecognized_drivers: i64
}

pub fn get_gpus() -> String {
    let instance = wgpu::Instance::default();
    let mut gpus: Vec<String> = vec![];
    let mut gpu_counter: HashMap<String, AdapterCnt> = HashMap::new();


    for a in instance.enumerate_adapters(wgpu::Backends::all()).iter().map(Adapter::get_info) {
        let amdgpu = Regex::new(r"(?i)AMD open-source driver").unwrap();
        let mesa = Regex::new("(?i)Mesa").unwrap();
        let vulkan = Regex::new("(?i)vulkan").unwrap();
        let gl = Regex::new("(?i)gl").unwrap();
        let mesa_name_fixer = Regex::new(r"(?i)\(.*(LLVM|DRM).*\)").unwrap();

        let name = mesa_name_fixer.replace(a.name.as_str(), "").trim().to_string();

        let mut driver: Option<String> = None;

        if amdgpu.is_match(a.driver.as_str()) {
            driver = Some("AMDgpu".to_string());
        } else if mesa.is_match(a.driver_info.as_str()) {
            driver = Some("Mesa".to_string());
        }

        if gpu_counter.contains_key(&name) {
            gpu_counter.get_mut(&name).unwrap().count += 1;
            gpu_counter.get_mut(&name).unwrap().gl |= gl.is_match(a.backend.to_str());
            gpu_counter.get_mut(&name).unwrap().vulkan |= vulkan.is_match(a.backend.to_str());
            if driver.is_none() {
                gpu_counter.get_mut(&name).unwrap().unrecognized_drivers += 1;
            } else {
                gpu_counter.get_mut(&name).unwrap().drivers.push(driver.unwrap());
            }
        } else {
            gpu_counter.insert(
                name.to_string(),
                AdapterCnt {
                    count: 1,
                    name: name.to_string(),
                    drivers: if driver.is_some() {
                        vec![driver.clone().unwrap()]
                    } else {
                        vec![]
                    },
                    unrecognized_drivers: if driver.is_some() {
                        0
                    } else {
                        1
                    },
                    vulkan: vulkan.is_match(a.backend.to_str()),
                    gl: gl.is_match(a.backend.to_str())
                },
            );
        }
    }

    for (_, gpu) in gpu_counter {
        let mut out = gpu.name + " (";

        if gpu.gl && gpu.vulkan {
            out += "Vulkan/OpenGL, ";
        } else if gpu.gl {
            out += "OpenGL, ";
        } else if gpu.vulkan {
            out += "Vulkan, ";
        }

        for driver in gpu.drivers {
            out += (driver + ", ").as_str();
        }

        if gpu.unrecognized_drivers > 1 {
            out += format!("{} unrecognized drivers)", gpu.unrecognized_drivers).as_str();
        } else if gpu.unrecognized_drivers == 1 {
            out += "1 unrecognized driver)";
        } else {
            out = out.trim_end_matches(", ").to_string() + ")";
        }

        gpus.push(out);
    }

    if gpus.len() == 0 {
        return "None... found?".to_string();
    } else if gpus.len() == 1 {
        return "╰ ".to_string() + gpus[0].as_str();
    } else {
        let mut out = "".to_string();
        for (i, mon) in gpus.iter().enumerate() {
            if i == gpus.len() - 1 {
                out.push_str("\n╰ ");
            } else {
                out.push_str("\n│ ");
            }
            out.push_str(mon.as_str());
        }
        return out.trim().to_string();
    }
}